# rtplot
A library for doing real-time plotting in Rust. This is still in its early
stages of development, so expect the API change and improve over time.

## Usage:

Usage of the library requires two steps: configuring the plot and displaying
the plot with its rendering function.

```
use rand_distr::{Distribution, Normal};
use rtplot::{Figure, PlotType};

fn main() {
    let normal = Normal::new(0.0, 1.0).unwrap();
    let mut rng = rand::thread_rng();
    // Configure the plot to hold 100 samples at a time. The x axis is not
    // specified, so it is dynamically updated at runtime.
    let mut figure = Figure::new(100)
        .ylim([-1.0, 1.0])
        .xlabel("Time (s)")
        .ylabel("Amplitude")
        .plot_type(PlotType::Line)
        .color(0x80, 0x00, 0x80);

    // The closure is called every time to generate a new state for the plot.
    // The figure _must_ be run on the main thread due to compatibility issues
    // with events loops on non-Unix operatins sytems.
    Figure::display(&mut figure, |fig| {
        let v: Vec<f32> = normal
            .sample_iter(&mut rng)
            .take(10)
            .map(|x| x as f32)
            .collect();
        fig.plot_stream(&v);
    });
}
```
