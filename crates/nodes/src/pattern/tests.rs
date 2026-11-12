use crate::{CookContext, find, resolve_params};
use materialcraft_core::{FunctionGraph, PatternGraph, PatternNodeKind, Value};
use std::collections::BTreeMap;

fn cook(graph: PatternGraph) -> crate::CookResult {
    cook_with_variables(graph, &BTreeMap::new())
}

fn cook_with_variables(graph: PatternGraph, variables: &BTreeMap<String, Value>) -> crate::CookResult {
    let definition = find("fx_map").unwrap();
    let parameters = resolve_params(definition, &BTreeMap::from([("pattern".into(), Value::PatternGraph(Box::new(graph)))]));
    let context = CookContext { w: 16, h: 16, seed: 7, tile: true, inputs: &[], params: &parameters, resource: &|_| None, vars: variables };
    (definition.cook)(&context)
}

#[test]
fn pattern_functions_keep_host_time_and_quadrant_depth() {
    for (name, expected) in [("$time", 0.75), ("$depth", 1.0), ("$depthpow2", 0.5)] {
        let mut graph = PatternGraph::with_root(PatternNodeKind::Quadrant);
        let child = graph.add(PatternNodeKind::Quadrant).unwrap();
        graph.nodes[0].parameters.insert("patterntype".into(), Value::Int(0));
        graph.nodes[0].children[0] = Some(child);
        graph.nodes[1].parameters.insert("patterntype".into(), Value::Int(2));
        graph.nodes[1].parameters.insert("patternsize".into(), Value::Float2([2.0; 2]));
        let mut function = FunctionGraph::default();
        function.output = Some(function.add("get_float1", &[], Some(Value::Text(name.into()))));
        graph.nodes[1].parameters.insert("opacity".into(), Value::Function(Box::new(function)));

        let image = cook_with_variables(graph, &BTreeMap::from([("$time".into(), Value::Float(0.75))])).unwrap().remove(0);

        assert!((image.mean()[0] - expected).abs() < 1e-6, "{name}: {}", image.mean()[0]);
    }
}

#[test]
fn iterate_sets_number_and_switch_selects_one_child() {
    let mut graph = PatternGraph::default();
    let iterate = graph.add(PatternNodeKind::Iterate).unwrap();
    let switch = graph.add(PatternNodeKind::Switch).unwrap();
    let quadrant = graph.add(PatternNodeKind::Quadrant).unwrap();
    graph.root = Some(iterate);
    graph.nodes[0].children = vec![None, Some(switch)];
    graph.nodes[0].parameters.insert("numberadded".into(), Value::Int(3));
    graph.nodes[1].children = vec![Some(quadrant), None];
    graph.nodes[2].parameters.insert("patterntype".into(), Value::Int(2));
    let mut function = FunctionGraph::default();
    function.output = Some(function.add("get_float1", &[], Some(Value::Text("$number".into()))));
    graph.nodes[2].parameters.insert("opacity".into(), Value::Function(Box::new(function)));
    let image = cook(graph.clone()).unwrap().remove(0);
    assert!((image.mean()[0] - 3.0).abs() < 1e-6);
    let dot = graph.add(PatternNodeKind::Dot).unwrap();
    graph.nodes[1].children[0] = Some(dot);
    graph.nodes[3].children[0] = Some(quadrant);
    assert_eq!(cook(graph.clone()).unwrap()[0].data, image.data);
    graph.nodes[1].parameters.insert("switch".into(), Value::Bool(true));
    assert_eq!(cook(graph).unwrap()[0].mean()[0], 0.0);
}

#[test]
fn iterate_preserves_assignments_from_previous_iterations() {
    let mut graph = PatternGraph::with_root(PatternNodeKind::Iterate);
    let child = graph.add(PatternNodeKind::Quadrant).unwrap();
    graph.nodes[0].children = vec![None, Some(child)];
    let mut initialize = FunctionGraph::default();
    let zero = initialize.add_constant(Value::Float(0.0));
    let count = initialize.add_constant(Value::Int(3));
    let set = initialize.add("set", &[zero], Some(Value::Text("counter".into())));
    initialize.output = Some(initialize.add("sequence", &[set, count], None));
    graph.nodes[0].parameters.insert("numberadded".into(), Value::Function(Box::new(initialize)));
    let mut increment = FunctionGraph::default();
    let value = increment.add("get_float1", &[], Some(Value::Text("counter".into())));
    let one = increment.add_constant(Value::Float(1.0));
    let next = increment.add("add", &[value, one], None);
    increment.output = Some(increment.add("set", &[next], Some(Value::Text("counter".into()))));
    graph.nodes[1].parameters.insert("patterntype".into(), Value::Int(2));
    graph.nodes[1].parameters.insert("opacity".into(), Value::Function(Box::new(increment)));
    let image = cook(graph).unwrap().remove(0);
    assert!(image.data.iter().all(|value| *value == 6.0), "expected 1 + 2 + 3, got {:?}", image.mean());
}

#[test]
fn patterns_reject_cycles_and_bound_exponential_traversal() {
    let mut graph = PatternGraph::default();
    let root = graph.add(PatternNodeKind::Quadrant).unwrap();
    graph.nodes[0].children[0] = Some(root);
    assert!(cook(graph).unwrap_err().contains("cycle"));

    let mut graph = PatternGraph::default();
    for _ in 0..12 {
        graph.add(PatternNodeKind::Quadrant).unwrap();
    }
    for index in 0..11 {
        graph.nodes[index].children = vec![Some(graph.nodes[index + 1].id); 4];
    }
    assert!(cook(graph).unwrap_err().contains("budget"));
}

#[test]
fn image_indices_outside_the_supported_range_return_an_error() {
    let mut graph = PatternGraph::with_root(PatternNodeKind::Quadrant);
    graph.nodes[0].parameters.insert("patterntype".into(), Value::Int(1));
    graph.nodes[0].parameters.insert("imageindex".into(), Value::Int(i32::MAX));
    assert!(cook(graph).unwrap_err().contains("unsupported pattern"));
}

fn roughness_sample(roughness: f32, depth: usize, blend: i32, color: bool, repeat: bool) -> crate::CookResult {
    let mut graph = PatternGraph::with_root(PatternNodeKind::Quadrant);
    for index in 0..depth {
        let child = graph.add(PatternNodeKind::Quadrant).unwrap();
        graph.nodes[index].children[0] = Some(child);
    }
    let leaf = depth;
    let leaf_id = graph.nodes[leaf].id;
    graph.nodes[leaf].parameters.insert("patterntype".into(), Value::Int(2));
    graph.nodes[leaf].parameters.insert("opacity".into(), Value::Float4([0.8, 0.4, 0.2, 1.0]));
    graph.nodes[leaf].parameters.insert("blendingmode".into(), Value::Int(blend));
    if repeat {
        let dot = graph.add(PatternNodeKind::Dot).unwrap();
        graph.nodes.last_mut().unwrap().children[0] = Some(leaf_id);
        let switch = graph.add(PatternNodeKind::Switch).unwrap();
        graph.nodes.last_mut().unwrap().children[0] = Some(dot);
        let iterate = graph.add(PatternNodeKind::Iterate).unwrap();
        graph.nodes.last_mut().unwrap().children = vec![None, Some(switch)];
        graph.nodes.last_mut().unwrap().parameters.insert("numberadded".into(), Value::Int(2));
        if leaf == 0 {
            graph.root = Some(iterate);
        } else {
            graph.nodes[leaf - 1].children[0] = Some(iterate);
        }
    }
    let definition = find("fx_map").unwrap();
    let parameters = resolve_params(
        definition,
        &BTreeMap::from([
            ("pattern".into(), Value::PatternGraph(Box::new(graph))),
            ("roughness".into(), Value::Float(roughness)),
            ("opacity".into(), Value::Float(0.5)),
            ("color_mode".into(), Value::Bool(color)),
            ("background".into(), Value::Color([0.1; 4])),
        ]),
    );
    let context = CookContext { w: 16, h: 16, seed: 7, tile: true, inputs: &[], params: &parameters, resource: &|_| None, vars: &BTreeMap::new() };
    (definition.cook)(&context)
}

#[test]
fn roughness_attenuates_quadrants_by_depth_and_keeps_background() {
    for color in [false, true] {
        for (roughness, depth, weight) in [(0.0, 2, 1.0), (1.0, 0, 1.0), (1.0, 1, 0.5), (0.5, 2, 1.0 / 7.0), (2.0, 2, 4.0 / 7.0)] {
            for blend in 0..=2 {
                let image = roughness_sample(roughness, depth, blend, color, false).unwrap().remove(0);
                let pixel = image.get(0, 0, false);
                for (channel, foreground) in [0.8_f32, 0.4, 0.2, 1.0].into_iter().take(image.kind.channels()).enumerate() {
                    let value = foreground * 0.5 * weight;
                    let expected = match blend {
                        1 if !color => value.max(0.1),
                        1 => value + 0.1 * (1.0 - 0.5 * weight),
                        2 => value.max(0.1),
                        _ => value + 0.1,
                    };
                    assert!((pixel[channel] - expected).abs() < 1e-6, "roughness={roughness}, depth={depth}, blend={blend}, channel={channel}: {pixel:?}");
                }
                if depth > 0 {
                    assert!((image.get(15, 15, false)[0] - 0.1).abs() < 1e-6);
                }
            }
        }
    }
}

#[test]
fn repetition_switches_and_dots_do_not_add_roughness_depth() {
    let image = roughness_sample(1.0, 2, 0, false, true).unwrap().remove(0);
    assert!((image.get(0, 0, false)[0] - (0.1 + 0.8 / 3.0)).abs() < 1e-6);
    assert!((image.get(15, 15, false)[0] - 0.1).abs() < 1e-6);
}

#[test]
fn negative_roughness_reports_an_error() {
    assert!(roughness_sample(-32.0, 5, 0, false, false).unwrap_err().contains("roughness must be finite and nonnegative"));
}

#[test]
fn a_full_nine_level_quadtree_renders_one_pattern_per_pixel() {
    let mut graph = PatternGraph::with_root(PatternNodeKind::Quadrant);
    for index in 0..8 {
        let child = graph.add(PatternNodeKind::Quadrant).unwrap();
        graph.nodes[index].children = vec![Some(child); 4];
    }
    graph.nodes.last_mut().unwrap().parameters.insert("patterntype".into(), Value::Int(2));
    let definition = find("fx_map").unwrap();
    let parameters = resolve_params(definition, &BTreeMap::from([("pattern".into(), Value::PatternGraph(Box::new(graph)))]));
    let context = CookContext { w: 256, h: 256, seed: 7, tile: true, inputs: &[], params: &parameters, resource: &|_| None, vars: &BTreeMap::new() };
    let images = (definition.cook)(&context).unwrap();
    assert!(images[0].data.iter().all(|value| (*value - 1.0).abs() < 1e-6));
}

fn square(graph: &mut PatternGraph, node: usize) {
    graph.nodes[node].parameters.insert("patterntype".into(), Value::Int(2));
}

#[test]
fn an_iteration_reads_the_variables_the_previous_iteration_set() {
    let mut graph = PatternGraph::default();
    let iterate = graph.add(PatternNodeKind::Iterate).unwrap();
    let quadrant = graph.add(PatternNodeKind::Quadrant).unwrap();
    graph.root = Some(iterate);
    graph.nodes[0].parameters.insert("numberadded".into(), Value::Int(3));
    graph.nodes[0].children[1] = Some(quadrant);
    square(&mut graph, 1);

    let mut function = FunctionGraph::default();
    let total = function.add("get_float1", &[], Some(Value::Text("total".into())));
    let step = function.add_constant(Value::Float(0.25));
    let next = function.add("add", &[total, step], None);
    function.output = Some(function.add("set", &[next], Some(Value::Text("total".into()))));
    graph.nodes[1].parameters.insert("opacity".into(), Value::Function(Box::new(function)));

    let image = cook_with_variables(graph, &BTreeMap::from([("total".into(), Value::Float(0.0))])).unwrap().remove(0);

    // 0.25, 0.5 and 0.75 add up; three separate copies of the variable would add 0.25 three times
    assert!((image.mean()[0] - 1.5).abs() < 1e-5, "{}", image.mean()[0]);
}

#[test]
fn the_render_region_is_the_part_of_pattern_space_the_image_shows() {
    let mut graph = PatternGraph::with_root(PatternNodeKind::Quadrant);
    square(&mut graph, 0);
    let definition = find("fx_map").unwrap();
    let parameters = resolve_params(
        definition,
        &BTreeMap::from([("pattern".into(), Value::PatternGraph(Box::new(graph))), ("render_region".into(), Value::Float4([0.0, 2.0, 0.0, 2.0]))]),
    );
    let context = CookContext { w: 16, h: 16, seed: 7, tile: false, inputs: &[], params: &parameters, resource: &|_| None, vars: &BTreeMap::new() };

    let image = (definition.cook)(&context).unwrap().remove(0);

    // The unit square fills the top-left quarter of a region twice as wide and tall
    assert!((image.mean()[0] - 0.25).abs() < 1e-5, "{}", image.mean()[0]);
    assert_eq!(image.get(3, 3, false)[0], 1.0);
    assert_eq!(image.get(12, 12, false)[0], 0.0);
}

#[test]
fn every_visit_draws_its_own_random_numbers() {
    let mut graph = PatternGraph::with_root(PatternNodeKind::Quadrant);
    graph.nodes[0].parameters.insert("patterntype".into(), Value::Int(0));
    let child = graph.add(PatternNodeKind::Quadrant).unwrap();
    for slot in 0..4 {
        graph.nodes[0].children[slot] = Some(child);
    }
    square(&mut graph, 1);
    let mut function = FunctionGraph::default();
    let range = function.add_constant(Value::Float(1.0));
    function.output = Some(function.add("rand", &[range], None));
    graph.nodes[1].parameters.insert("opacity".into(), Value::Function(Box::new(function)));

    let image = cook(graph).unwrap().remove(0);

    let quarters = [image.get(4, 4, false)[0], image.get(12, 4, false)[0], image.get(4, 12, false)[0], image.get(12, 12, false)[0]];
    assert!(quarters.windows(2).all(|pair| (pair[0] - pair[1]).abs() > 1e-4), "{quarters:?}");
}

#[test]
fn a_pixel_on_the_seam_of_two_patterns_is_drawn_once() {
    let mut graph = PatternGraph::with_root(PatternNodeKind::Quadrant);
    graph.nodes[0].parameters.insert("patterntype".into(), Value::Int(0));
    let child = graph.add(PatternNodeKind::Quadrant).unwrap();
    let grandchild = graph.add(PatternNodeKind::Quadrant).unwrap();
    for slot in 0..4 {
        graph.nodes[0].children[slot] = Some(child);
        graph.nodes[1].children[slot] = Some(grandchild);
    }
    graph.nodes[1].parameters.insert("patterntype".into(), Value::Int(0));
    square(&mut graph, 2);
    let definition = find("fx_map").unwrap();
    let parameters = resolve_params(definition, &BTreeMap::from([("pattern".into(), Value::PatternGraph(Box::new(graph)))]));
    let variables = BTreeMap::new();
    // Two pixels a side put every pixel center on the corner of four of the sixteen squares
    let context = CookContext { w: 2, h: 2, seed: 7, tile: true, inputs: &[], params: &parameters, resource: &|_| None, vars: &variables };

    let image = (definition.cook)(&context).unwrap().remove(0);

    assert_eq!(image.get(0, 0, false)[0], 1.0);
    assert_eq!(image.get(1, 1, false)[0], 1.0);
}
