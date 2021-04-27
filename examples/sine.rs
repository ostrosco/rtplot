use itertools_num::linspace;
use rtplot::{Figure, PlotType};
use std::f32::consts::PI;

fn calculate_sin(amplitude: f32, phase: f32) -> Vec<f32> {
    let sin_vals: Vec<_> = linspace(0.0, 100.0, 10000)
        .map(|x| amplitude * (PI / 8.0 * x as f32 + phase).sin())
        .collect();

    sin_vals
}

fn main() {
    let mut phase = 0.0;
    let mut figure = Figure::new(10000)
        .xlabel("Time (s)")
        .ylabel("Amplitude")
        .plot_type(PlotType::Line)
        .color(0xFF, 0x00, 0x00);

    Figure::display(&mut figure, |fig| {
        let sin_vals = calculate_sin(10.0, phase);
        fig.plot_y(&sin_vals);
        phase += PI / 20.0;
    });
}
