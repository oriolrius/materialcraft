//! Library previews: a small image of what a library graph makes from sample inputs

use std::collections::BTreeMap;
use std::sync::Arc;

use materialcraft_core::Package;
use materialcraft_image::{Image, Kind};

use crate::{NodeLibrary, Session};

/// Smallest spread between an image's darkest and brightest sample that still shows something
const FLAT_SPREAD: f32 = 0.01;
/// Side in pixels a graph is cooked at before it is scaled down to the preview
const COOKED_SIZE: u32 = 128;

impl NodeLibrary {
    /// What graph `graph` of package `package` makes at `size` pixels, from a wave height, its
    /// colors and its normal map on the graph's inputs. `None` when the graph has no output that
    /// cooks or its image is one flat value
    pub fn preview(library: &Arc<NodeLibrary>, package: &str, graph: &str, size: u32) -> Option<Image> {
        let source = library.packages.get(package)?;
        let index = source.graphs.iter().position(|candidate| candidate.function.is_none() && candidate.sbs_path() == graph)?;

        // A session links the graph's instances and functions to the library, as an opened package gets
        let session = Session::with_library(Package::clone(source), library.clone());
        let graph = session.pkg.graphs.get(index)?;
        let decode = session.resource_decoder();
        let mut options = materialcraft_eval::Options::new(&decode);
        // Thin lines and fine grain vanish when cooked at thumbnail size, so the graph cooks larger and is scaled down
        let cooked_size = size.max(COOKED_SIZE);
        options.max_size = Some(cooked_size);
        options.inputs = sample_inputs(graph, cooked_size);
        let cooked = materialcraft_eval::cook(&session.pkg, index, &options, &session.cache);

        let outputs = materialcraft_eval::outputs(graph, &cooked);
        let base_color = outputs.iter().find(|(_, usage, _, image)| *usage == "baseColor" && image.is_some());
        let image = base_color.or_else(|| outputs.iter().find(|output| output.3.is_some()))?.3?;
        (!is_flat(image)).then(|| image.resize(size.min(image.w), size.min(image.h), false))
    }
}

/// An image for every input node of `graph`, by identifier: normals for normal inputs, colors for color inputs, the height for the rest
fn sample_inputs(graph: &materialcraft_core::Graph, size: u32) -> BTreeMap<String, Image> {
    let height = Image::generate_uv(size, size, Kind::Gray, |u, v| {
        [0.5 + 0.25 * (u * std::f32::consts::TAU).sin() + 0.25 * (v * std::f32::consts::TAU).cos(), 0.0, 0.0, 0.0]
    });
    let color = height.map(Kind::Color, |pixel| [0.15 + 0.7 * pixel[0], 0.35 + 0.3 * pixel[0], 0.8 - 0.5 * pixel[0], 1.0]);
    let normal = materialcraft_nodes::atomic::height_to_normal(&height, 4.0, false, true, false);

    graph
        .nodes
        .iter()
        .filter(|node| matches!(node.def.as_str(), "input_color" | "input_grayscale"))
        .map(|node| {
            let identifier = materialcraft_eval::identifier(node);
            let image = if identifier.to_ascii_lowercase().contains("normal") {
                &normal
            } else if node.def == "input_color" {
                &color
            } else {
                &height
            };
            (identifier, image.clone())
        })
        .collect()
}

/// Whether every pixel of `image` has nearly the same value
fn is_flat(image: &Image) -> bool {
    let mut low = [f32::INFINITY; 3];
    let mut high = [f32::NEG_INFINITY; 3];
    for y in 0..i64::from(image.h) {
        for x in 0..i64::from(image.w) {
            let pixel = image.get(x, y, false);
            for channel in 0..3 {
                low[channel] = low[channel].min(pixel[channel]);
                high[channel] = high[channel].max(pixel[channel]);
            }
        }
    }
    (0..3).all(|channel| high[channel] - low[channel] < FLAT_SPREAD)
}
