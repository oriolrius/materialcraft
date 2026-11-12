use super::{By, MaterialCraftApp, Queryable, UiState, harness, json, save, settle};
use egui::accesskit::Role;

fn directory() -> std::path::PathBuf {
    let directory = std::path::PathBuf::from(std::env::var_os("MC_SNAPSHOT_DIR").unwrap());
    std::fs::create_dir_all(&directory).unwrap();
    directory
}

#[test]
#[ignore = "requires an offscreen GPU; run explicitly with --ignored"]
fn function_nodes_show_their_values_and_symbols() {
    let mut app = MaterialCraftApp::headless(materialcraft_engine::new_session(), UiState::default());
    app.run("file.new", json!({"kind": "function", "name": "Look"})).unwrap();
    let operations = [
        ("const_float1", [0, 0]),
        ("const_float2", [0, 120]),
        ("const_float4", [0, 260]),
        ("const_int1", [0, 420]),
        ("const_bool", [0, 520]),
        ("get_float1", [160, 0]),
        ("add", [160, 120]),
        ("mul", [160, 240]),
        ("div", [160, 360]),
        ("gt", [320, 0]),
        ("sqrt", [320, 120]),
        ("lerp", [320, 240]),
        ("ifelse", [320, 380]),
        ("vector3", [480, 0]),
        ("swizzle2", [480, 120]),
        ("sin", [480, 240]),
        ("min", [480, 360]),
    ];
    for (operation, at) in operations {
        app.run("graph.function.edit", json!({"edit": {"action": "add", "operation": operation, "at": at}})).unwrap();
    }
    let mut harness = harness(app, 1600.0, 960.0);
    settle(&mut harness, 8);

    save(&mut harness, &directory(), "look-function-nodes");
}

#[test]
#[ignore = "requires an offscreen GPU; run explicitly with --ignored"]
fn library_nodes_and_tiles_show_previews() {
    let mut app = MaterialCraftApp::headless(materialcraft_engine::new_session(), UiState::default());
    let entries = [
        ("node:uniform", [0, 0]),
        ("node:blur", [160, 0]),
        ("sbs://blur_hq.sbs#blur_hq", [320, 0]),
        ("sbs://edge_detect.sbs#edge_detect", [480, 0]),
        ("sbs://materials/pbr/bricks_001.sbs#bricks_001", [0, 220]),
        ("sbs://pattern_tile_sampler.sbs#tile_sampler", [200, 220]),
        ("sbs://mg_mask_builder.sbs#mg_mask_builder_2", [400, 220]),
        ("node:input_value", [640, 0]),
        ("node:value_processor", [640, 100]),
    ];
    for (entry, at) in entries {
        app.run("library.add", json!({"entry": entry, "at": at})).unwrap();
    }
    let mut harness = harness(app, 1600.0, 960.0);
    // The graph cooks on a worker
    for _ in 0..480 {
        std::thread::sleep(std::time::Duration::from_millis(250));
        settle(&mut harness, 2);
        let session = &harness.state().session;
        if session.graph().is_ok_and(|graph| session.cooked.nodes.len() >= graph.nodes.len()) {
            break;
        }
    }
    save(&mut harness, &directory(), "look-library-nodes");

    for (top, sub, name) in
        [("Filters", "Blurs", "look-library-blurs"), ("PBR materials", "Bricks", "look-library-bricks"), ("Functions", "Constant", "look-library-functions")]
    {
        harness.get(By::new().role(Role::TreeItem).label(top)).click();
        settle(&mut harness, 3);
        harness.get_all(By::new().role(Role::TreeItem).label(sub)).next().unwrap().click();
        // Previews cook on a worker
        for _ in 0..40 {
            std::thread::sleep(std::time::Duration::from_millis(250));
            settle(&mut harness, 2);
        }
        save(&mut harness, &directory(), name);
        harness.get(By::new().role(Role::TreeItem).label(top)).click();
        settle(&mut harness, 2);
        harness.get(By::new().role(Role::TreeItem).label(top)).click();
        settle(&mut harness, 2);
    }
}
