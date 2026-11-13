use super::*;
use materialcraft_core::{Graph, GraphInstance, Package, Resource, ResourceBytes};
use std::sync::Arc;

fn library() -> NodeLibrary {
    let mut library = NodeLibrary::default();
    let mut root = Package::new("root");
    let mut graph = Graph::new("main");
    let id = graph.add_node("graph_instance", [0.0, 0.0]);
    graph.node_mut(id).unwrap().instance = Some(GraphInstance {
        label: String::new(),
        package: "child".into(),
        graph: "sub".into(),
        inputs: vec![],
        outputs: vec![],
        defaults: Default::default(),
    });
    root.graphs.push(graph);
    let mut child = Package::new("child");
    child.graphs.push(Graph::new("sub"));
    child.resources.push(Resource::new("bitmap", "bitmap.png", ResourceBytes::new(vec![1, 2, 3])));
    library.packages.insert("root".into(), Arc::new(root));
    library.packages.insert("child".into(), Arc::new(child));
    library
}

fn key(library: &NodeLibrary) -> Option<[u8; 32]> {
    library.preview_fingerprint("root", "main", &AtomicBool::new(false))
}

#[test]
fn keys_survive_reloading_and_track_nested_content_not_unrelated_packages() {
    let original = library();
    assert_eq!(key(&original), key(&library()));
    let mut changed = library();
    changed.packages.insert("unrelated".into(), Arc::new(Package::new("unused")));
    assert_eq!(key(&original), key(&changed));
    Arc::make_mut(changed.packages.get_mut("child").unwrap()).graphs[0].seed += 1;
    assert_ne!(key(&original), key(&changed));
    let mut changed = library();
    Arc::make_mut(changed.packages.get_mut("child").unwrap()).resources[0].bytes = ResourceBytes::new(vec![1, 2, 4]);
    assert_ne!(key(&original), key(&changed), "same-size replacement must invalidate the preview");
}

#[test]
fn incomplete_and_cancelled_keys_are_not_persisted() {
    let mut library = library();
    assert!(library.preview_fingerprint("root", "main", &AtomicBool::new(true)).is_none());
    library.packages.remove("child");
    assert!(key(&library).is_none());
}
