use materialcraft_engine::{
    Session,
    core::{FunctionGraph, Graph, Package, Resource, ResourceBytes, Value},
    execute, file_cmds,
    formats::sbs,
};
use serde_json::json;

const TILE: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" width="2" height="2"><rect width="2" height="2" fill="red"/></svg>"#;

#[test]
fn bundled_voronoi_bitmaps_load_the_shared_resource_and_survive_portable_saving() {
    let path = "sbs://noise_voronoi.sbs";
    let bytes = sbs::library::read(path).unwrap();
    let read = sbs::read_sbs(&bytes, path, &file_cmds::linked_files(path)).unwrap();
    assert!(!read.warnings.iter().any(|warning| warning.contains("resource")), "{:?}", read.warnings);
    let bitmaps: Vec<_> = read.pkg.graphs.iter().flat_map(|graph| &graph.nodes).filter(|node| node.def == "bitmap").cloned().collect();
    assert_eq!(bitmaps.len(), 4);
    let expected = sbs::library::read("sbs://resources/perm.png").unwrap();
    for bitmap in &bitmaps {
        let Value::Text(name) = &bitmap.params["resource"] else { panic!("missing resource") };
        let resource = read.pkg.resource(name).unwrap();
        assert_eq!(resource.bytes.as_slice(), expected);
        assert_eq!(resource.source, "resources/perm.png");
    }
    // Exercise the repaired bitmaps independently of remaining Voronoi function limitations.
    let mut package = Package::new("Resource regression");
    package.resources = read.pkg.resources;
    let mut graph = Graph::new("Bitmaps");
    for mut bitmap in bitmaps {
        bitmap.id = graph.add_node("bitmap", [0.0; 2]);
        *graph.nodes.last_mut().unwrap() = bitmap;
    }
    package.graphs.push(graph);
    let mut session = Session::with_library(package, std::sync::Arc::new(Default::default()));
    session.preview_size = Some(64);
    let cooked = session.cook().clone();
    for node in &session.pkg.graphs[0].nodes {
        assert!(cooked.nodes[&node.id].error.is_none());
        let image = cooked.image(node.id, 0).unwrap();
        let low = image.data.iter().copied().fold(f32::INFINITY, f32::min);
        let high = image.data.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        assert!(low.is_finite() && high.is_finite() && high > low);
    }
    let saved = file_cmds::save_bytes(&session).unwrap();
    let reopened = sbs::read_sbs(&saved, "relocated.sbs", &|_| None).unwrap();
    assert!(reopened.warnings.is_empty(), "{:?}", reopened.warnings);
    assert_eq!(reopened.pkg.resources[0].bytes.as_slice(), expected);
}

fn resource_package(source: &str) -> Package {
    let mut package = Package::new("Resource resolution");
    let mut graph = Graph::new("Constant");
    graph.function = Some(FunctionGraph::constant(Value::Float(0.5)));
    package.graphs.push(graph);
    let mut resource = Resource::new("Tile", "tile.svg", ResourceBytes::new(Vec::new()));
    resource.source = source.into();
    package.resources.push(resource);
    package
}

#[test]
fn library_resources_resolve_next_to_their_package() {
    const CHILD_ROOT: &str = "MC_LINKED_RESOURCES_TEST_ROOT";
    if let Ok(root) = std::env::var(CHILD_ROOT) {
        verify_library_resources(std::path::Path::new(&root));
        return;
    }

    // A child process isolates the library root without mutating the test runner's environment
    let root = std::env::temp_dir().join(format!("materialcraft-linked-resources-{}", std::process::id()));
    std::fs::create_dir_all(root.join("assets/sbs/resources")).unwrap();
    std::fs::create_dir_all(root.join("assets/sbs/nested/resources")).unwrap();
    std::fs::write(root.join("assets/sbs/resources/tile.svg"), TILE).unwrap();
    std::fs::write(root.join("assets/sbs/nested/resources/tile.svg"), TILE).unwrap();
    let package = sbs::write_sbs(&resource_package("resources/tile.svg")).unwrap();
    std::fs::write(root.join("assets/sbs/functions.sbs"), &package).unwrap();
    std::fs::write(root.join("assets/sbs/nested/source.sbs"), &package).unwrap();
    std::fs::write(
        root.join("assets/sbs/nested/caller.sbs"),
        r#"<package><dependencies>
        <dependency><filename v="source.sbs"/><uid v="7"/></dependency></dependencies><content>
        <function><identifier v="Caller"/><type v="256"/><paramValue><dynamicValue><rootnode v="1"/><paramNodes>
        <paramNode><uid v="1"/><function v="instance"/><type v="256"/><funcDatas><funcData><name v="instance"/>
        <constantValueString v="pkg:///Constant?dependency=7"/></funcData></funcDatas></paramNode>
        </paramNodes></dynamicValue></paramValue></function></content></package>"#,
    )
    .unwrap();

    let status = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "linked_resources::library_resources_resolve_next_to_their_package", "--nocapture"])
        .env(CHILD_ROOT, &root)
        .env("MATERIALCRAFT_SBS_DIR", root.join("assets/sbs"))
        .status()
        .unwrap();
    std::fs::remove_dir_all(root).unwrap();

    assert!(status.success());
}

fn verify_library_resources(root: &std::path::Path) {
    let library =
        materialcraft_engine::NodeLibrary::load(&["sbs://nested/caller.sbs".into(), "sbs://nested/source.sbs".into(), "sbs://nested/source.sbs".into()]);
    assert!(library.warnings.is_empty(), "{:?}", library.warnings);
    assert_eq!(library.packages.len(), 2);
    assert!(library.packages.values().all(|package| package.dependencies.is_empty()), "shared sources must not be retained again inside each package");
    let mut session = Session::new(Package::new("Caller"));
    let library = session.pkg.dependencies.get("sbs://functions.sbs").unwrap();
    assert_eq!(library.resources[0].bytes.as_slice(), TILE);

    file_cmds::open_file(&mut session, "sbs://nested/source.sbs").unwrap();
    assert!(session.load_warnings.is_empty(), "{:?}", session.load_warnings);
    assert_eq!(session.pkg.resources[0].bytes.as_slice(), TILE);
    execute(&mut session, "file.reload", &json!({})).unwrap();
    assert_eq!(session.pkg.resources[0].bytes.as_slice(), TILE);

    let read = file_cmds::linked_files("sbs://nested/source.sbs");
    assert_eq!(read("resources/tile.svg").unwrap(), TILE);
    assert_eq!(read("sbs://resources/tile.svg").unwrap(), TILE);
    assert_eq!(read("../resources/tile.svg").unwrap(), TILE);
    assert!(read("../../private.svg").is_none());
    assert!(read("resources/absent.svg").is_none());

    // Native filesystem opens use the same shared root as alias opens, including nested packages.
    let native = root.join("assets/sbs/nested/source.sbs");
    let native_read = file_cmds::linked_files(native.to_str().unwrap());
    assert_eq!(native_read("../resources/tile.svg").unwrap(), TILE);
    assert!(native_read("../../../outside.svg").is_none());
    let destination = root.join("assets/sbs/nested/saved.sbs");
    let linked = file_cmds::save_files_at(&session, destination.to_str().unwrap()).unwrap();
    assert_eq!(linked.files.len(), 1, "a library save retains links instead of copying resource bytes");
    let xml = linked.xml().unwrap();
    assert!(!std::str::from_utf8(xml).unwrap().contains("sbs://"));
    let restored = sbs::read_sbs(xml, destination.to_str().unwrap(), &file_cmds::linked_files(destination.to_str().unwrap())).unwrap();
    assert!(restored.warnings.is_empty(), "{:?}", restored.warnings);
    assert_eq!(restored.pkg.resources[0].bytes.as_slice(), TILE);

    // Saving embeds loaded resources so a relocated package remains self-contained
    let saved = file_cmds::save_bytes(&session).unwrap();
    let reopened = sbs::read_sbs(&saved, "elsewhere/copy.sbs", &|_| None).unwrap();
    assert!(reopened.warnings.is_empty(), "{:?}", reopened.warnings);
    assert_eq!(reopened.pkg.resources[0].bytes.as_slice(), TILE);

    let source_document = session.document_id;
    execute(&mut session, "file.newPackage", &json!({})).unwrap();
    execute(&mut session, "file.new", &json!({"kind": "function"})).unwrap();
    execute(&mut session, "graph.instance.add", &json!({"document": source_document, "graph": "Constant"})).unwrap();
    let caller = file_cmds::save_bytes(&session).unwrap();
    let mut dependent = Session::new(Package::new("Dependent"));
    file_cmds::open_bytes(&mut dependent, &caller, Some(root.join("dependent.sbs").to_string_lossy().into_owned())).unwrap();
    assert_eq!(dependent.pkg.dependencies["sbs://nested/source.sbs"].resources[0].bytes.as_slice(), TILE);

    let missing = sbs::write_sbs(&resource_package("resources/absent.svg")).unwrap();
    file_cmds::open_bytes_linked(&mut session, &missing, Some("sbs://nested/missing.sbs".into()), &read).unwrap();
    assert!(session.pkg.resources[0].bytes.is_empty());
    assert_eq!(session.pkg.resources[0].source, "sbs://nested/resources/absent.svg");
    assert!(!session.load_warnings.is_empty());

    let local = root.join("caller.sbs");
    std::fs::write(&local, sbs::write_sbs(&resource_package("sbs://resources/tile.svg")).unwrap()).unwrap();
    file_cmds::open_file(&mut session, local.to_str().unwrap()).unwrap();
    assert_eq!(session.pkg.resources[0].bytes.as_slice(), TILE);
}
