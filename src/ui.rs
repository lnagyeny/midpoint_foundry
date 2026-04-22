use crate::algorithms::Algorithm;
use crate::benchmark::{run_benchmark, BenchmarkStats};

// ── UI State ─────────────────────────────────────────────────────────────────

#[derive(Default)]
pub struct UiState {
    pub benchmark_radius: i32,
    pub benchmark_results: Option<Vec<(String, BenchmarkStats)>>,
    pub selected_category: String,
}

// ── UI Panel Drawing ─────────────────────────────────────────────────────────

/// Returns `true` if the current algorithm needs to be recomputed.
pub fn draw_panel(
    ctx: &egui::Context,
    ui_state: &mut UiState,
    algorithms: &mut Vec<Box<dyn Algorithm>>,
    selected: &mut usize,
    grid_size: &mut i32,
    selected_cell: Option<(i32, i32)>,
) -> bool {
    let mut needs_recompute = false;

    const UI_W: f32 = 350.0; // right-panel pixel width

    egui::SidePanel::right("control_panel")
        .exact_width(UI_W)
        .resizable(false)
        .show(ctx, |ui| {
            ui.heading("Algorithm Control");
            ui.separator();

            // ── Category selector ─────────────────────────────────────────
            let categories: Vec<&str> = {
                let mut cats: Vec<&str> = algorithms.iter().map(|a| a.category()).collect();
                cats.sort();
                cats.dedup();
                cats
            };

            if ui_state.selected_category.is_empty() {
                ui_state.selected_category = categories[0].to_owned();
            }

            egui::ComboBox::from_label("Category")
                .selected_text(&ui_state.selected_category)
                .show_ui(ui, |ui| {
                    for cat in &categories {
                        if ui
                            .selectable_value(
                                &mut ui_state.selected_category,
                                cat.to_string(),
                                *cat,
                            )
                            .changed()
                        {
                            if let Some(idx) = algorithms
                                .iter()
                                .position(|a| a.category() == ui_state.selected_category)
                            {
                                *selected = idx;
                                algorithms[*selected].compute();
                            }
                        }
                    }
                });

            // ── Algorithm selector ────────────────────────────────────────
            let filtered: Vec<(usize, String)> = algorithms
                .iter()
                .enumerate()
                .filter(|(_, a)| a.category() == ui_state.selected_category)
                .map(|(i, a)| (i, a.name().to_owned()))
                .collect();

            let current_name = algorithms[*selected].name().to_owned();
            let prev = *selected;

            egui::ComboBox::from_label("Algorithm")
                .selected_text(&current_name)
                .show_ui(ui, |ui| {
                    for (i, name) in &filtered {
                        ui.selectable_value(selected, *i, name.as_str());
                    }
                });

            if *selected != prev {
                algorithms[*selected].compute();
            }

            ui.separator();

            // ── Grid size ─────────────────────────────────────────────────
            ui.add(
                egui::Slider::new(grid_size, 10..=200)
                    .text("Grid size")
                    .step_by(2.0),
            );

            ui.separator();

            // ── Algorithm-specific controls ───────────────────────────────
            let param_changed = algorithms[*selected].draw_ui(ui);
            if param_changed {
                needs_recompute = true;
            }

            ui.separator();

            if ui.button("⟳  Recompute").clicked() {
                needs_recompute = true;
            }

            ui.separator();

            // ── Benchmark settings ────────────────────────────────────────
            ui.add(
                egui::Slider::new(&mut ui_state.benchmark_radius, 1..=100).text("Benchmark Radius"),
            );

            if ui.button("📊 Benchmark").clicked() {
                ui_state.benchmark_results = Some(run_benchmark(ui_state.benchmark_radius));
            }

            // ── Benchmark results ────────────────────────────────────────
            if let Some(results) = &ui_state.benchmark_results {
                ui.separator();

                ui.horizontal(|ui| {
                    ui.heading("Benchmark Results (µs)");
                    let text = results
                        .iter()
                        .map(|(name, s)| {
                            format!(
                                "{}\nMin:    {}\nMax:    {}\nAvg:    {}\nMedian: {}",
                                name, s.min_us, s.max_us, s.avg_us, s.median_us
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("\n\n");

                    if ui.button("📋 Copy").clicked() {
                        ui.output_mut(|o| o.copied_text = text);
                    }
                });

                egui::ScrollArea::vertical()
                    .max_height(400.0)
                    .show(ui, |ui| {
                        for (algo_name, stats) in results {
                            ui.separator();
                            ui.label(algo_name);
                            ui.monospace(format!("Min:    {}", stats.min_us));
                            ui.monospace(format!("Max:    {}", stats.max_us));
                            ui.monospace(format!("Avg:    {}", stats.avg_us));
                            ui.monospace(format!("Median: {}", stats.median_us));
                        }
                    });
            }

            // ── Selected cell info ────────────────────────────────────────
            if let Some((cx, cy)) = selected_cell {
                ui.separator();
                ui.label("Selected cell");
                ui.monospace(format!("X: {cx},  Y: {cy}"));

                if let Some(info) = algorithms[*selected].cell_info(cx, cy) {
                    ui.monospace(info);
                }
            }

            ui.separator();
            ui.label(format!("{} points", algorithms[*selected].points().len()));
        });

    needs_recompute
}
