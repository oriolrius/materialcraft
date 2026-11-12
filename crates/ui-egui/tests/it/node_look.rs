//! The Library's draggable divider and the faces of function nodes

use egui::accesskit::Role;
use egui::{Event, Modifiers, PointerButton, Pos2, vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::{By, Queryable};
use materialcraft_ui_egui::{MaterialCraftApp, UiState};
use serde_json::json;

fn harness(app: MaterialCraftApp) -> Harness<'static, MaterialCraftApp> {
    // Short steps keep two clicks within the double-click delay
    let mut harness = Harness::builder().with_size(vec2(1600.0, 960.0)).with_pixels_per_point(1.0).with_step_dt(0.05).build_ui_state(
        |ui, app: &mut MaterialCraftApp| {
            app.step(&ui.ctx().clone());
            app.draw(ui);
        },
        app,
    );
    harness.run_steps(6);
    harness
}

fn press(harness: &mut Harness<'static, MaterialCraftApp>, at: Pos2, pressed: bool) {
    harness.event(Event::PointerButton { pos: at, button: PointerButton::Primary, pressed, modifiers: Modifiers::NONE });
    harness.step();
}

fn divider(harness: &Harness<'static, MaterialCraftApp>) -> Pos2 {
    harness.get(By::new().role(Role::Splitter).label("Library divider")).rect().center()
}

#[test]
fn the_library_divider_drags_the_tree_narrower_and_a_double_click_resets_it() {
    let mut harness = harness(MaterialCraftApp::headless(materialcraft_engine::new_session(), UiState::default()));
    assert_eq!(harness.state().ui.library_tree_width, 0.0, "the tree starts at its automatic width");
    let start = divider(&harness);

    harness.event(Event::PointerMoved(start));
    harness.step();
    press(&mut harness, start, true);
    for step in 1..=6 {
        harness.event(Event::PointerMoved(start - vec2(10.0 * step as f32, 0.0)));
        harness.step();
    }
    press(&mut harness, start - vec2(60.0, 0.0), false);
    harness.run_steps(3);

    let dragged = harness.state().ui.library_tree_width;
    assert!(dragged > 0.0, "a drag chooses a width");
    let moved = start.x - divider(&harness).x;
    assert!((40.0..=62.0).contains(&moved), "the divider follows the pointer, moved {moved}");

    let at = divider(&harness);
    harness.event(Event::PointerMoved(at));
    harness.step();
    for pressed in [true, false, true, false] {
        press(&mut harness, at, pressed);
    }
    harness.run_steps(3);
    assert_eq!(harness.state().ui.library_tree_width, 0.0, "a double click returns to the automatic width");
    assert!((divider(&harness).x - start.x).abs() < 1.0);
}

#[test]
fn a_constant_node_fits_a_field_for_each_component() {
    let mut app = MaterialCraftApp::headless(materialcraft_engine::new_session(), UiState::default());
    app.run("file.new", json!({"kind": "function", "name": "Faces"})).unwrap();
    app.run("graph.function.edit", json!({"edit": {"action": "add", "operation": "const_float4", "at": [0, 0]}})).unwrap();
    app.run("graph.function.edit", json!({"edit": {"action": "add", "operation": "add", "at": [200, 0]}})).unwrap();
    let harness = harness(app);

    let card = harness.get(By::new().role(Role::Button).label("Float4")).rect();
    for component in ["X", "Y", "Z", "W"] {
        let field = harness.get(By::new().role(Role::SpinButton).label(component)).rect();
        assert!(card.contains_rect(field), "{component} lies inside its node: {field:?} in {card:?}");
    }

    let operator = harness.get(By::new().role(Role::Button).label("Add")).rect();
    assert!(operator.height() < card.height(), "an operator is a compact node");
    assert!(operator.height() < operator.width(), "an operator has no empty preview square");
}
