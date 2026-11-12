use materialcraft_core::{PatternGraph, PatternNodeKind, Value};
use materialcraft_nodes::{CookContext, find, resolve_params};
use std::collections::BTreeMap;

fn render(region: [f32; 4], tiling: [f32; 4], tile: bool) -> materialcraft_nodes::CookResult {
    let mut graph = PatternGraph::with_root(PatternNodeKind::Quadrant);
    graph.nodes[0].parameters.insert("patterntype".into(), Value::Int(2));
    graph.nodes[0].parameters.insert("patternsize".into(), Value::Float2([0.5; 2]));
    let definition = find("fx_map").unwrap();
    let params = resolve_params(
        definition,
        &BTreeMap::from([
            ("pattern".into(), Value::PatternGraph(Box::new(graph))),
            ("render_region".into(), Value::Float4(region)),
            ("tiling_region".into(), Value::Float4(tiling)),
        ]),
    );
    (definition.cook)(&CookContext { w: 64, h: 64, seed: 0, tile, inputs: &[], params: &params, resource: &|_| None, vars: &BTreeMap::new() })
}

#[test]
fn render_region_samples_a_cropped_area() {
    let identity = [0.0, 1.0, 0.0, 1.0];
    let full = render(identity, identity, false).unwrap().remove(0);
    let crop = render([0.25, 0.75, 0.25, 0.75], identity, false).unwrap().remove(0);
    assert_eq!(full.mean()[0], 0.25);
    assert!(crop.data.iter().all(|value| *value == 1.0));
}

#[test]
fn tiling_region_sets_repeat_distance_independently_of_render_region() {
    let region = [0.0, 2.0, 0.0, 1.0];
    let repeated = render(region, [0.0, 1.0, 0.0, 1.0], true).unwrap().remove(0);
    let wider = render(region, region, true).unwrap().remove(0);
    let untiled = render(region, [0.0, 1.0, 0.0, 1.0], false).unwrap().remove(0);
    assert_eq!(repeated.get(48, 32, false)[0], 1.0);
    assert_eq!(wider.get(48, 32, false)[0], 0.0);
    assert_eq!(wider.data, untiled.data);
    assert_eq!(repeated.mean()[0], 0.25);
    assert_eq!(wider.mean()[0], 0.125);
}

#[test]
fn degenerate_regions_and_excessive_repetition_return_errors() {
    let identity = [0.0, 1.0, 0.0, 1.0];
    assert!(render([0.0; 4], identity, true).is_err());
    assert!(render(identity, [0.0; 4], true).is_err());
    assert!(render(identity, [0.0, 1e-9, 0.0, 1e-9], true).is_err());
}
