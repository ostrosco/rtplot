use itertools_num::linspace;
use rtplot::{Figure, PlotType};
use std::f32::consts::PI;
use std::thread;

fn calculate_sin(phase: f32) -> Vec<f32> {
    let sin_vals: Vec<_> = linspace(0.0, 100.0, 10000)
        .map(|x| 10.0 * (PI / 8.0 * x as f32 + phase).sin())
        .collect();

    sin_vals
}

fn main() {
    let mut phase = 0.0;
    let handle = thread::spawn(move || {
        let mut figure = Figure::new()
            .init_renderer(10000)
            .xlim([-0.5, 0.5])
            .ylim([-10.0, 10.0])
            .xlabel("Time (s)")
            .ylabel("Amplitude")
            .plot_type(PlotType::Dot)
            .color(0x80, 0x00, 0x80);

        Figure::display(&mut figure, |fig| {
            let sin_vals = calculate_sin(phase);
            fig.plot_y(&sin_vals);
            phase += PI / 20.0;
        });
    });

    handle.join().unwrap();
}
