use materialcraft_core::{FunctionGraph, PatternGraph, PatternNodeKind, Value};
use materialcraft_image::{Image, Kind};
use materialcraft_nodes::{CookContext, find, resolve_params};
use std::collections::BTreeMap;

fn cook(def: &str, values: BTreeMap<String, Value>, inputs: &[Option<Image>]) -> Image {
    let definition = find(def).unwrap();
    let params = resolve_params(definition, &values);
    let context = CookContext {
        format: materialcraft_core::OutputFormat::F32,
        w: 64,
        h: 64,
        seed: 0,
        tile: true,
        inputs,
        params: &params,
        resource: &|_| None,
        vars: &BTreeMap::new(),
    };
    (definition.cook)(&context).unwrap().remove(0)
}

#[test]
fn hsl_hue_endpoints_complete_a_full_turn() {
    let color = [0.7, 0.3, 0.1, 0.6];
    let inputs = [Some(Image::filled(64, 64, Kind::Color, color))];
    for hue in [0.0, 0.5, 1.0] {
        let image = cook("hsl", BTreeMap::from([("hue".into(), Value::Float(hue))]), &inputs);
        for (actual, expected) in image.get(0, 0, false).into_iter().zip(color) {
            assert!((actual - expected).abs() < 1e-5, "hue {hue}: {actual} != {expected}");
        }
    }
    let image = cook("hsl", BTreeMap::from([("hue".into(), Value::Float(0.75))]), &inputs);
    assert!((image.get(0, 0, false)[2] - 0.7).abs() < 1e-5);
}

#[test]
fn repeated_quadrants_have_distinct_reproducible_random_values() {
    let mut graph = PatternGraph::with_root(PatternNodeKind::Quadrant);
    let child = graph.add(PatternNodeKind::Quadrant).unwrap();
    graph.nodes[0].children = vec![Some(child); 4];
    graph.nodes[1].parameters.insert("patterntype".into(), Value::Int(2));
    let mut function = FunctionGraph::default();
    let one = function.add("const_float1", &[], Some(Value::Float(1.0)));
    function.output = Some(function.add("rand", &[one], None));
    graph.nodes[1].parameters.insert("opacity".into(), Value::Function(Box::new(function)));
    let values = BTreeMap::from([("pattern".into(), Value::PatternGraph(Box::new(graph)))]);
    let image = cook("fx_map", values.clone(), &[]);
    let samples: Vec<_> = [(16, 16), (48, 16), (16, 48), (48, 48)].map(|(x, y)| image.get_gray(x, y, false)).into_iter().collect();
    assert!(samples.windows(2).all(|pair| pair[0] != pair[1]), "{samples:?}");
    assert_eq!(image.data, cook("fx_map", values, &[]).data);
}

#[test]
fn brick_variation_bevels_both_axes_without_narrowing_the_plateau_to_a_strip() {
    let mut graph = PatternGraph::with_root(PatternNodeKind::Quadrant);
    graph.nodes[0].parameters.insert("patterntype".into(), Value::Int(9));
    graph.nodes[0].parameters.insert("patternsuppl".into(), Value::Float(0.5));
    let image = cook("fx_map", BTreeMap::from([("pattern".into(), Value::PatternGraph(Box::new(graph)))]), &[]);
    let center = image.get_gray(32, 32, false);
    let horizontal_edge = image.get_gray(3, 32, false);
    let vertical_edge = image.get_gray(32, 3, false);
    assert_eq!(center, 1.0);
    assert!(horizontal_edge > 0.0 && horizontal_edge < center);
    assert!((horizontal_edge - vertical_edge).abs() < 1e-5);
}

#[test]
fn subpixel_quadrants_do_not_accumulate_at_shared_edges() {
    let mut graph = PatternGraph::with_root(PatternNodeKind::Quadrant);
    for index in 0..7 {
        let child = graph.add(PatternNodeKind::Quadrant).unwrap();
        graph.nodes[index].children = vec![Some(child); 4];
    }
    let leaf = graph.nodes.last_mut().unwrap();
    leaf.parameters.insert("patterntype".into(), Value::Int(2));
    leaf.parameters.insert("opacity".into(), Value::Float(0.25));
    let image = cook("fx_map", BTreeMap::from([("pattern".into(), Value::PatternGraph(Box::new(graph)))]), &[]);
    assert!(image.data.iter().all(|value| (*value - 0.25).abs() < 1e-6), "mean {}", image.mean()[0]);
}

#[test]
fn grayscale_max_patterns_do_not_accumulate_opacity() {
    let mut graph = PatternGraph::with_root(PatternNodeKind::Iterate);
    let child = graph.add(PatternNodeKind::Quadrant).unwrap();
    graph.nodes[0].children[1] = Some(child);
    graph.nodes[0].parameters.insert("numberadded".into(), Value::Int(1));
    graph.nodes[1].parameters.insert("patterntype".into(), Value::Int(4));
    graph.nodes[1].parameters.insert("blendingmode".into(), Value::Int(1));
    let values = |graph| BTreeMap::from([("pattern".into(), Value::PatternGraph(Box::new(graph)))]);
    let single = cook("fx_map", values(graph.clone()), &[]);
    graph.nodes[0].parameters.insert("numberadded".into(), Value::Int(2));
    let repeated = cook("fx_map", values(graph), &[]);
    assert!(single.data == repeated.data, "repeating a maximum blend must not brighten the image");
}
