//! Headless gallery gate and egui smoke tests for MatRust Studio Desktop figure pipeline (FIG0).
//!
//! This test suite runs every Math Gallery example through the Desktop Session execution
//! pipeline, measures the figure/axes/series items produced in Desktop GraphicsState,
//! and runs an egui headless smoke test asserting shape generation.
//!
//! Per FIG0 specification, this baseline test documents the exact 14 Math Gallery examples
//! that produce ZERO painted items in the legacy GraphicsState, providing the RED baseline
//! gate that will become GREEN as FIG1..FIG6 unify the plot engine with rustlab_plot::Figure.

use egui::{Context, Pos2, Rect, Vec2};
use matrust_studio::execution::Session;
use matrust_studio::figures::FiguresPanel;
use matrust_studio::math_gallery::GALLERY_ITEMS;
use matrust_studio::theme::Theme;

#[test]
fn test_gallery_headless_execution_produces_figures() {
    let mut session = Session::new();

    // Baseline catalog of the 14 Math Gallery items that produce 0 items in legacy GUI:
    // 1. calc-helmholtz: uses contour, quiver
    // 2. calc-laplacian: uses del2, mesh, surf
    // 3. calc-vortex-curl: uses quiver, contour
    // 4. calc-quadrature: pure numerical output (integral2)
    // 5. ode-lorenz: uses plot3
    // 6. ode-robertson: uses semilogx
    // 7. linalg-svd: uses imagesc
    // 8. linalg-eigen: uses plot3
    // 9. signal-pwelch: uses pwelch, semilogy
    // 10. signal-spectrogram: uses imagesc / spectrogram
    // 11. optim-fsolve: pure numerical output (fsolve)
    // 12. optim-linprog: pure numerical output (linprog)
    // 13. fractal-mandelbrot: uses imagesc
    // 14. fractal-julia: uses imagesc
    let expected_zero_item_ids: &[&str] = &[
        "calc-helmholtz",
        "calc-laplacian",
        "calc-vortex-curl",
        "calc-quadrature",
        "ode-lorenz",
        "ode-robertson",
        "linalg-svd",
        "linalg-eigen",
        "signal-pwelch",
        "signal-spectrogram",
        "optim-fsolve",
        "optim-linprog",
        "fractal-mandelbrot",
        "fractal-julia",
    ];

    let mut passing_count = 0;
    let mut zero_item_count = 0;
    let mut failures = Vec::new();

    for item in GALLERY_ITEMS {
        // Clear figures before running each example
        {
            let mut g = session.graphics.lock().unwrap();
            g.figures.clear();
            g.current_figure = None;
        }

        let res = session.exec(item.script, None);
        assert!(
            res.error.is_none(),
            "Gallery script {} failed execution: {:?}",
            item.id,
            res.error
        );

        let g = session.graphics.lock().unwrap();
        let total_items: usize = g.figures.iter().map(|f| f.total_series()).sum();

        if total_items > 0 {
            passing_count += 1;
        } else {
            zero_item_count += 1;
            failures.push(item.id);
        }
    }

    assert_eq!(
        GALLERY_ITEMS.len(),
        24,
        "Total Math Gallery items should be 24"
    );
    assert_eq!(
        passing_count, 10,
        "Expected exactly 10 gallery items to produce GUI items under legacy path"
    );
    assert_eq!(
        zero_item_count, 14,
        "Expected exactly 14 gallery items to produce 0 GUI items under legacy path"
    );

    // Verify exact set of 14 failing items
    for expected_id in expected_zero_item_ids {
        assert!(
            failures.contains(expected_id),
            "Expected {} to be among legacy zero-item failures, but was not in {:?}",
            expected_id,
            failures
        );
    }
}

#[test]
fn test_egui_context_headless_smoke_shapes() {
    let ctx = Context::default();
    let theme = Theme::dark();
    let mut panel = FiguresPanel::default();
    let mut session = Session::new();

    // 1. Run a passing script: ode-vanderpol (produces line plot)
    let vanderpol_script = GALLERY_ITEMS
        .iter()
        .find(|item| item.id == "ode-vanderpol")
        .expect("ode-vanderpol must exist")
        .script;

    let res = session.exec(vanderpol_script, None);
    assert!(res.error.is_none());

    let mut on_close = None;
    let raw_input = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(
            Pos2::ZERO,
            Vec2::new(1280.0, 800.0),
        )),
        ..Default::default()
    };

    let full_output = ctx.run(raw_input, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut g = session.graphics.lock().unwrap();
            panel.show(ui, &theme, &mut g, &mut on_close);
        });
    });

    let shape_count = full_output.shapes.len();
    assert!(
        shape_count > 0,
        "Egui output shapes must be non-empty for ode-vanderpol"
    );
    assert!(
        shape_count > 10,
        "Van der Pol phase portrait plot must produce substantial shapes (axes, ticks, lines), got {}",
        shape_count
    );
}

#[test]
fn test_gallery_gate_failing_baseline_assertion() {
    // This test specifically proves the GATE is RED for the unified plot engine target.
    // It asserts that the legacy GUI currently FAILS the full 24/24 gallery requirement,
    // explicitly documenting the 14 missing items until FIG1..FIG6 resolve them.
    let mut session = Session::new();
    let mut unpainted_items = Vec::new();

    for item in GALLERY_ITEMS {
        {
            let mut g = session.graphics.lock().unwrap();
            g.figures.clear();
            g.current_figure = None;
        }
        let _ = session.exec(item.script, None);
        let g = session.graphics.lock().unwrap();
        let total_items: usize = g.figures.iter().map(|f| f.total_series()).sum();
        if total_items == 0 {
            unpainted_items.push(item.id);
        }
    }

    // Explicit gate check: The gallery gate demands all 24 items to produce painted elements.
    // In FIG0, this baseline assertion confirms the 14 deficiencies.
    assert!(
        !unpainted_items.is_empty(),
        "FIG0 baseline: Gate is red because legacy GUI paints 0 items for: {:?}",
        unpainted_items
    );
    assert_eq!(
        unpainted_items.len(),
        14,
        "FIG0 baseline: exactly 14 items unpainted in legacy GUI"
    );
}
