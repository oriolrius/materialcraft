use materialcraft_core::{PatternGraph, PatternNodeKind, Value};
use materialcraft_nodes::{CookContext, find, resolve_params};
use std::collections::BTreeMap;

fn render(depth: usize, roughness: f32, stack: bool, color: Option<[f32; 4]>) -> materialcraft_image::Image {
    let mut graph = PatternGraph::with_root(PatternNodeKind::Quadrant);
    for index in 0..depth {
        let child = graph.add(PatternNodeKind::Quadrant).unwrap();
        graph.nodes[index].children.fill(Some(child));
    }
    for (index, node) in graph.nodes.iter_mut().enumerate() {
        node.parameters.insert("patterntype".into(), Value::Int(if stack || index == depth { 2 } else { 0 }));
        node.parameters.insert("opacity".into(), Value::Float4(color.unwrap_or([1.0; 4])));
    }
    let definition = find("fx_map").unwrap();
    let params = resolve_params(
        definition,
        &BTreeMap::from([
            ("pattern".into(), Value::PatternGraph(Box::new(graph))),
            ("roughness".into(), Value::Float(roughness)),
            ("opacity".into(), Value::Float(0.125)),
            ("color_mode".into(), Value::Bool(color.is_some())),
            ("background".into(), Value::Color([0.0; 4])),
        ]),
    );
    let context = CookContext { w: 64, h: 64, seed: 0, tile: true, inputs: &[], params: &params, resource: &|_| None, vars: &BTreeMap::new() };
    (definition.cook)(&context).unwrap().remove(0)
}

#[test]
fn quadrant_opacity_matches_measured_depth_and_stack_values() {
    // Pixel values measured from an original constant-square SBS diagnostic at 32-bit float precision.
    // Empty ancestor quadrants still count toward the depth normalization; zero roughness bypasses it.
    for (roughness, depth, expected) in [
        (0.0, 0, 0.125),
        (0.0, 1, 0.125),
        (0.0, 4, 0.125),
        (0.42, 0, 0.125),
        (0.42, 1, 0.036971834),
        (0.42, 4, 0.0022858537),
        (0.5, 0, 0.125),
        (0.5, 1, 0.041666668),
        (0.5, 4, 0.004032258),
        (1.0, 0, 0.125),
        (1.0, 1, 0.0625),
        (1.0, 4, 0.025),
    ] {
        let image = render(depth, roughness, false, None);
        assert!(
            image.data.iter().all(|value| (*value - expected).abs() < 2e-7),
            "roughness {roughness}, depth {depth}: {:?}, expected {expected}",
            image.mean()
        );
    }
    for roughness in [0.42, 1.0] {
        let image = render(4, roughness, true, None);
        assert!(image.data.iter().all(|value| (*value - 0.125).abs() < 2e-7), "stack roughness {roughness}: {:?}", image.mean());
    }
}

#[test]
fn colored_patterns_premultiply_rgb_by_their_opacity_alpha() {
    let image = render(0, 0.0, false, Some([0.4, 0.2, 0.1, 0.5]));
    for (actual, expected) in image.get(32, 32, false).into_iter().zip([0.025, 0.0125, 0.00625, 0.0625]) {
        assert!((actual - expected).abs() < 1e-7, "{actual} != {expected}");
    }
}
