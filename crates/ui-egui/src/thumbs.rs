use std::collections::{BTreeMap, HashMap};

use egui::{ColorImage, TextureHandle, TextureOptions};
use materialcraft_engine::image::{Image, Kind};
use materialcraft_engine::nodes::{CookContext, catalog, resolve_params};

/// Rendered previews by id: (id, RGBA8 pixels).
pub type Previews = Vec<(String, Vec<u8>)>;

/// Side of node thumbnails and library previews, in pixels
pub const THUMB: u32 = 96;
const LIBRARY_PREVIEW_SIZE: u32 = 72;

#[derive(Default)]
pub struct Thumbs {
    /// (cook key, port) → texture and the frame it was last used
    nodes: HashMap<(u64, usize), (TextureHandle, u64)>,
    #[cfg(not(target_arch = "wasm32"))]
    persistence: Option<preview_cache_settings::Maintenance>,
    clear_epoch: u64,
    graphs: GraphPreviews,
    frame: u64,
}

/// An image as an egui texture image (sRGB display, `side` px)
pub fn color_image(img: &Image, side: u32) -> ColorImage {
    let small = img.resize(side, side, true);
    ColorImage::from_rgba_unmultiplied([side as usize, side as usize], &small.to_rgba8_display())
}

#[path = "graph_previews.rs"]
mod graph_previews;
use graph_previews::GraphPreviews;
#[cfg(not(target_arch = "wasm32"))]
#[path = "preview_cache_settings.rs"]
mod preview_cache_settings;
#[cfg(not(target_arch = "wasm32"))]
#[path = "preview_disk_cache.rs"]
mod preview_disk_cache;

impl Thumbs {
    /// Opt in only for a native app window; offscreen harnesses never use the user's cache.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn enable_persistence(&mut self, prefs: materialcraft_engine::LibraryCachePrefs) {
        let persistence = preview_cache_settings::Maintenance::new(prefs);
        self.graphs.persistence = Some(persistence.settings.clone());
        self.persistence = Some(persistence);
    }

    pub fn configure_cache(&mut self, _prefs: materialcraft_engine::LibraryCachePrefs, clear_epoch: u64) {
        let clear = clear_epoch != self.clear_epoch;
        self.clear_epoch = clear_epoch;
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(persistence) = &self.persistence {
            persistence.settings.update(_prefs.into(), clear);
        }
        if clear {
            self.graphs.clear();
        }
    }

    pub fn clear_nodes(&mut self) {
        self.nodes.clear();
    }

    /// The thumbnail of a node output, uploaded on first use
    pub fn node(&mut self, ctx: &egui::Context, key: u64, port: usize, img: &Image) -> egui::TextureId {
        let frame = self.frame;
        if let Some((t, f)) = self.nodes.get_mut(&(key, port)) {
            *f = frame;
            return t.id();
        }
        let tex = ctx.load_texture(format!("node-{key}-{port}"), color_image(img, THUMB), TextureOptions::LINEAR);
        let id = tex.id();
        self.nodes.insert((key, port), (tex, frame));
        id
    }

    /// Once per frame: forget thumbnails unused for a few seconds; collect library previews
    pub fn tick(&mut self, ctx: &egui::Context, library: &std::sync::Arc<materialcraft_engine::NodeLibrary>, foreground_busy: bool) -> Option<String> {
        self.frame += 1;
        let f = self.frame;
        if self.nodes.len() > 256 {
            self.nodes.retain(|_, (_, used)| f - *used < 120);
        }
        #[cfg(not(target_arch = "wasm32"))]
        let error = self.persistence.as_mut().and_then(|persistence| persistence.tick(ctx));
        #[cfg(target_arch = "wasm32")]
        let error = None;
        self.graphs.collect(ctx, library, foreground_busy);
        error
    }

    /// The preview of library graph `graph` of `package`, cooked in the background on first use;
    /// `None` until it is ready and for graphs that make no image
    pub fn graph(
        &mut self,
        ctx: &egui::Context,
        library: &std::sync::Arc<materialcraft_engine::NodeLibrary>,
        package: &str,
        graph: &str,
    ) -> Option<egui::TextureId> {
        self.graphs.get(ctx, library, package, graph)
    }

    /// The library preview of node `def` (generated in the background on first use)
    pub fn library(&mut self, ctx: &egui::Context, def: &str) -> Option<egui::TextureId> {
        self.graphs.native(ctx, def)
    }
}

/// Sample inputs for previews: a wave height, a colored version and its normal map
fn sample_inputs() -> (Image, Image, Image) {
    let gray = Image::generate_uv(LIBRARY_PREVIEW_SIZE, LIBRARY_PREVIEW_SIZE, Kind::Gray, |u, v| {
        [0.5 + 0.25 * (u * std::f32::consts::TAU).sin() + 0.25 * (v * std::f32::consts::TAU).cos(), 0.0, 0.0, 0.0]
    });
    let color = gray.map(Kind::Color, |p| [0.15 + 0.7 * p[0], 0.35 + 0.3 * p[0], 0.8 - 0.5 * p[0], 1.0]);
    let normal = materialcraft_engine::nodes::atomic::height_to_normal(&gray, 4.0, false, true, false);
    (gray, color, normal)
}

fn render_native(id: &str) -> Option<Vec<u8>> {
    let def = catalog().iter().find(|def| def.id == id && def.id != "graph_instance")?;
    let (gray, color, normal) = sample_inputs();
    let no_res = |_: &str| None;
    let vars = BTreeMap::new();
    let inputs: Vec<Option<Image>> = def
        .inputs
        .iter()
        .enumerate()
        .map(|(i, p)| {
            // Masks and secondary inputs stay empty so the main effect shows.
            if i > 0 && !matches!(def.id, "blend" | "height_blend" | "normal_blend" | "rgba_merge") {
                return None;
            }
            Some(if p.id.contains("normal") || def.category.contains("Normal") && p.kind == materialcraft_engine::nodes::PortKind::Color {
                normal.clone()
            } else if p.kind == materialcraft_engine::nodes::PortKind::Color {
                color.clone()
            } else if i == 1 {
                gray.map(Kind::Gray, |v| [1.0 - v[0], 0.0, 0.0, 0.0])
            } else {
                gray.clone()
            })
        })
        .collect();
    let params = resolve_params(def, &BTreeMap::new());
    let ctx = CookContext {
        format: materialcraft_engine::core::OutputFormat::U16,
        w: LIBRARY_PREVIEW_SIZE,
        h: LIBRARY_PREVIEW_SIZE,
        seed: 3,
        tile: true,
        inputs: &inputs,
        params: &params,
        resource: &no_res,
        vars: &vars,
    };
    let imgs = (def.cook)(&ctx).ok()?;
    Some(imgs.first()?.resize(LIBRARY_PREVIEW_SIZE, LIBRARY_PREVIEW_SIZE, true).to_rgba8_display())
}
