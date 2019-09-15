use rand::distributions::{Distribution, Normal};
use rtplot::{Figure, PlotType};
use std::thread;

fn main() {
    let handle = thread::spawn(move || {
        let normal = Normal::new(0.0, 1.0);
        let mut rng = rand::thread_rng();
        let mut figure = Figure::new()
            .init_window(100)
            .ylim([-1.0, 1.0])
            .xlabel("Time (s)")
            .ylabel("Amplitude")
            .plot_type(PlotType::Line)
            .color(0x80, 0x00, 0x80);

        Figure::display(&mut figure, |fig| {
            let v: Vec<f32> = normal
                .sample_iter(&mut rng)
                .take(10)
                .map(|x| x as f32)
                .collect();
            fig.plot_stream(&v);
        });
    });

    handle.join().unwrap();
}
