use materialcraft_engine::{
    Session, atomic_sample,
    core::{FunctionGraph, Graph, Package, PatternGraph, PatternNodeKind, Value},
    execute,
    formats::sbs::{read_sbs, write_sbs},
};
use serde_json::json;

#[test]
fn pattern_edits_are_atomic_undoable_and_preserve_native_layout() {
    let mut session = Session::new(atomic_sample().unwrap());
    let owner = session.graph().unwrap().nodes.iter().find(|node| node.def == "fx_map").unwrap().id;
    let original = session.pkg.clone();
    let Value::PatternGraph(pattern) = &session.graph().unwrap().node(owner).unwrap().params["pattern"] else { panic!("missing pattern") };
    let mut pattern: PatternGraph = (**pattern).clone();
    pattern.nodes[2].position = [256.0, 512.0];
    let dot = pattern.add(PatternNodeKind::Dot).unwrap();
    pattern.nodes[0].children[1] = Some(dot);
    pattern.nodes[3].children[0] = Some(pattern.nodes[2].id);
    execute(&mut session, "graph.pattern.set", &json!({"id": owner.0, "pattern": pattern})).unwrap();
    let edited = session.pkg.clone();
    let history = session.history();
    let root = pattern.root.unwrap();
    pattern.nodes[2].children[0] = Some(root);
    assert!(execute(&mut session, "graph.pattern.set", &json!({"id": owner.0, "pattern": pattern})).is_err());
    assert_eq!(session.pkg, edited);
    assert_eq!(session.history(), history);
    session.undo().unwrap();
    assert_eq!(session.pkg, original);
    session.redo().unwrap();
    assert_eq!(session.pkg, edited);

    let loaded = read_sbs(&write_sbs(&session.pkg).unwrap(), "pattern.sbs", &|_| None).unwrap();
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    let node = loaded.pkg.graphs[0].nodes.iter().find(|node| node.def == "fx_map").unwrap();
    let Value::PatternGraph(graph) = &node.params["pattern"] else { panic!("missing pattern") };
    assert_eq!(graph.nodes[2].position, [256.0, 512.0]);
    assert_eq!(graph.root, Some(graph.nodes[0].id));
    assert_eq!(graph.nodes[0].children, vec![Some(graph.nodes[1].id), Some(graph.nodes[3].id)]);
    assert_eq!(graph.nodes[1].children, vec![None, Some(graph.nodes[2].id)]);
    assert_eq!(graph.nodes[3].kind, PatternNodeKind::Dot);
    assert_eq!(graph.nodes[3].children, vec![Some(graph.nodes[2].id)]);
    assert!(matches!(graph.nodes[2].parameters["frameoffset"], Value::Function(_)));
}

#[test]
fn dynamic_roughness_survives_native_saving_and_invalidates_the_cook_cache() {
    let mut pattern = PatternGraph::with_root(PatternNodeKind::Quadrant);
    let child = pattern.add(PatternNodeKind::Quadrant).unwrap();
    pattern.nodes[0].children[0] = Some(child);
    pattern.nodes[1].parameters.insert("patterntype".into(), Value::Int(2));
    let mut function = FunctionGraph::default();
    function.output = Some(function.add("get_float1", &[], Some(Value::Text("$time".into()))));
    let mut graph = Graph::new("Depth");
    graph.size = [4, 4];
    let owner = graph.add_node("fx_map", [0.0; 2]);
    let node = graph.node_mut(owner).unwrap();
    node.params.insert("pattern".into(), Value::PatternGraph(Box::new(pattern)));
    node.functions.insert("roughness".into(), function);
    let mut package = Package::new("Depth");
    package.graphs.push(graph);

    for _ in 0..2 {
        let saved = write_sbs(&package).unwrap();
        assert!(std::str::from_utf8(&saved).unwrap().contains("<name v=\"hurst\"/>"));
        let loaded = read_sbs(&saved, "roughness.sbs", &|_| None).unwrap();
        assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
        package = loaded.pkg;
        let owner = package.graphs[0].nodes[0].id;
        let cache = std::sync::Mutex::new(materialcraft_engine::eval::Cache::default());
        let mut options = materialcraft_engine::eval::Options::new(&|_| None);
        for (time, expected) in [(0.0, 1.0), (1.0, 0.5), (2.0, 0.25)] {
            options.environment.time_seconds = time;
            let result = materialcraft_engine::eval::cook(&package, 0, &options, &cache);
            assert_eq!(result.nodes[&owner].error, None);
            assert!(!result.nodes[&owner].cached);
            assert!((result.image(owner, 0).unwrap().get(0, 0, false)[0] - expected).abs() < 1e-6);
            let cached = materialcraft_engine::eval::cook(&package, 0, &options, &cache);
            assert!(cached.nodes[&owner].cached);
            assert_eq!(cached.image(owner, 0).unwrap().data, result.image(owner, 0).unwrap().data);
        }
    }
}
