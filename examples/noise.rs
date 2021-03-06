use rand_distr::{Distribution, Normal};
use rtplot::{Figure, PlotType};

fn main() {
    let normal = Normal::new(0.0, 1.0).unwrap();
    let mut rng = rand::thread_rng();
    let mut figure = Figure::new(100)
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
}
