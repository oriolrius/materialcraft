use materialcraft_engine::{
    Session,
    core::{FunctionGraph, Graph, Package},
    execute,
};
use serde_json::{Value, json};

#[test]
fn bundled_cloud_and_white_noise_parameter_scopes_cook() {
    let mut session = materialcraft_engine::new_session();
    session.preview_size = Some(64);
    for file in ["clouds_3.sbs", "noise_clouds_3_v2.sbs", "white_noise_v2.sbs"] {
        materialcraft_engine::file_cmds::open_file(&mut session, &format!("sbs://{file}")).unwrap();
        let warnings = materialcraft_engine::function_inspection::warnings(session.graph().unwrap());
        assert!(warnings.is_empty(), "{file}: {warnings:?}");
        let nodes = session.graph().unwrap().nodes.clone();
        let cooked = session.cook();
        let failures: Vec<_> = cooked.nodes.iter().filter_map(|(id, result)| result.error.as_ref().map(|error| (*id, error))).collect();
        assert!(failures.is_empty(), "{file}: {failures:?}");
        for node in nodes.iter().filter(|node| matches!(node.def.as_str(), "output" | "fx_map")) {
            let image = cooked.image(node.id, 0).unwrap();
            let low = image.data.iter().copied().fold(f32::INFINITY, f32::min);
            let high = image.data.iter().copied().fold(f32::NEG_INFINITY, f32::max);
            assert!(high - low > 0.05, "{file} lost variation: {low}..{high}");
        }
    }
}

#[path = "../support/node_library.rs"]
pub(super) mod fixture;

#[test]
fn bundled_brick_material_cooks_without_missing_functions_or_budget_errors() {
    let mut session = materialcraft_engine::new_session();
    session.preview_size = Some(64);
    materialcraft_engine::file_cmds::open_file(&mut session, "sbs://materials/pbr/bricks_001.sbs").unwrap();
    let outputs: Vec<_> = session.graph().unwrap().nodes.iter().filter(|node| node.def == "output").map(|node| node.id).collect();
    assert!(!outputs.is_empty());
    let cooked = session.cook();
    let failures: Vec<_> = cooked.nodes.iter().filter_map(|(id, result)| result.error.as_ref().map(|error| (*id, error))).collect();
    assert!(failures.is_empty(), "{failures:?}");
    for output in outputs {
        let image = cooked.image(output, 0).unwrap();
        assert!(image.data.iter().all(|value| value.is_finite()));
    }
    // Finite output alone missed flat noise, identical tile values and a half-turn hue error.
    for id in [24, 34, 48, 91] {
        let image = cooked.image(materialcraft_engine::core::NodeId(id), 0).unwrap();
        let low = image.data.iter().copied().fold(f32::INFINITY, f32::min);
        let high = image.data.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        assert!(high - low > 0.05, "node {id} lost its variation: {low}..{high}");
        if id == 24 {
            assert!((0.15..0.85).contains(&image.mean()[0]), "fractal noise has lost its neutral range");
        }
    }
    let color = cooked.image(materialcraft_engine::core::NodeId(99), 0).unwrap().mean();
    assert!(color[0] > color[2], "the default brick color shifted to blue: {color:?}");
}

/// Return the catalog entries exposed by the library command.
fn entries(session: &mut Session, parameters: Value) -> Vec<Value> {
    execute(session, "library.tree", &parameters).unwrap()["entries"].as_array().unwrap().clone()
}

/// Catalog filters preserve matching entries and reject unknown canvases.
#[test]
fn the_library_tree_lists_categories_and_filters_entries() {
    let mut session = fixture::session();
    let tree = execute(&mut session, "library.tree", &json!({})).unwrap();
    let names: Vec<_> = tree["categories"].as_array().unwrap().iter().map(|category| category["name"].as_str().unwrap()).collect();
    assert_eq!(names[..4], ["Graph items", "Atomic nodes", "FxMap nodes", "Function nodes"]);
    assert!(names.contains(&"MCF functions") && !names.contains(&"SDF functions"));
    assert_eq!(tree["layouts"], json!([]));

    let constants = entries(&mut session, json!({"category": "Functions/Constant"}));
    assert!(constants.iter().any(|entry| entry["label"] == "Pi"));
    let functions = entries(&mut session, json!({"category": "Functions"}));
    let expected = fixture::library().packages["sbs://functions.sbs"].graphs.iter().filter(|graph| graph.function.is_some()).count();
    assert_eq!(functions.len(), expected, "every bundled function, across the sub-categories");
    assert!(entries(&mut session, json!({})).iter().all(|entry| entry["missing"].is_null()));
    assert!(entries(&mut session, json!({"category": "Texture generators"})).is_empty());

    let functions = entries(&mut session, json!({"canvas": "function", "filter": "pi"}));
    assert!(functions.iter().any(|entry| entry["id"] == "sbs://functions.sbs#Functions/Math/Pi" && entry["type"] == "functionInstance"));
    assert!(functions.iter().all(|entry| entry["canvas"] == "function" || entry["canvas"] == "any"));
    assert!(execute(&mut session, "library.tree", &json!({"canvas": "teapot"})).is_err());
}

/// Library additions use compatible canvases and participate in undo.
#[test]
fn library_entries_add_to_the_graph_they_fit() {
    let mut session = fixture::session();
    let before = session.graph().unwrap().nodes.len();
    let added = execute(&mut session, "library.add", &json!({"entry": "node:blur", "at": [32.0, 64.0]})).unwrap();
    assert_eq!(added["entry"], "node:blur");
    let graph = session.graph().unwrap();
    assert_eq!(graph.nodes.len(), before + 1);
    assert_eq!(graph.nodes.last().unwrap().def, "blur");
    assert_eq!(graph.nodes.last().unwrap().pos, [32.0, 64.0]);

    for (entry, check) in [("item:comment", "comments"), ("item:frame", "frames"), ("item:pin", "pins")] {
        let count = |session: &Session| serde_json::to_value(session.graph().unwrap()).unwrap()[check].as_array().map_or(0, Vec::len);
        let previous = count(&session);
        execute(&mut session, "library.add", &json!({"entry": entry})).unwrap();
        assert_eq!(count(&session), previous + 1, "{entry}");
    }

    assert!(execute(&mut session, "library.add", &json!({"entry": "nothing"})).is_err());
    assert!(execute(&mut session, "library.add", &json!({})).is_err());

    // Function entries need a Function canvas
    assert!(execute(&mut session, "library.add", &json!({"entry": "function:add"})).is_err());
    assert!(execute(&mut session, "library.add", &json!({"entry": "pattern:paramset"})).is_err());

    let mut package = Package::new("Functions");
    let mut graph = Graph::new("Caller");
    graph.function = Some(FunctionGraph::default());
    package.graphs.push(graph);
    let mut session = Session::with_library(package, fixture::library());
    execute(&mut session, "library.add", &json!({"entry": "function:add", "at": [0.0, 0.0]})).unwrap();
    execute(&mut session, "library.add", &json!({"entry": "sbs://functions.sbs#Functions/Math/Pi", "at": [140.0, 0.0]})).unwrap();
    let function = session.graph().unwrap().function.as_ref().unwrap();
    assert_eq!(function.nodes.len(), 2);
    assert!(function.nodes.iter().any(|node| node.op == "add"));
    assert!(function.nodes.iter().any(|node| node.instance.as_ref().is_some_and(|instance| instance.graph == "Functions/Math/Pi")));
    assert!(execute(&mut session, "library.add", &json!({"entry": "node:blur"})).is_err());
    execute(&mut session, "edit.undo", &json!({})).unwrap();
    assert_eq!(session.graph().unwrap().function.as_ref().unwrap().nodes.len(), 1);
}

/// Reload reports the installed catalog without modifying an unrelated graph.
#[test]
fn reloading_the_library_keeps_open_work_unmodified() {
    let mut session = fixture::session();
    let reloaded = execute(&mut session, "library.reload", &json!({})).unwrap();
    let entries = session.library_entries();
    assert_eq!(reloaded["entries"].as_u64(), Some(entries.len() as u64));
    assert_eq!(reloaded["available"].as_u64(), Some(entries.iter().filter(|entry| entry.missing.is_none()).count() as u64));
    assert!(!session.modified);
    assert!(session.library.packages.keys().all(|path| session.pkg.dependencies.contains_key(path)));
}

/// Reload must retain an explicitly supplied package even when its alias matches the library.
#[test]
fn review_reload_preserves_user_library_alias() {
    let mut dependency = Package::new("User functions");
    let mut function = Graph::new("User_function");
    function.function = Some(FunctionGraph::default());
    dependency.graphs.push(function);
    let mut package = Package::new("Project");
    package.graphs.push(Graph::new("Material"));
    package.dependencies.insert("sbs://functions.sbs".into(), std::sync::Arc::new(dependency.clone()));
    let mut session = Session::with_library(package, fixture::library());

    execute(&mut session, "library.reload", &json!({})).unwrap();

    assert!(session.pkg.dependencies["sbs://functions.sbs"].graphs == dependency.graphs, "the user dependency was replaced");
    assert!(entries(&mut session, json!({})).iter().any(|entry| entry["id"] == "sbs://functions.sbs#User_function"));
}
