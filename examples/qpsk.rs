use num::Complex;
use rand::seq::SliceRandom;
use rand_distr::{Distribution, Normal};
use rtplot::{Figure, PlotType};
use std::f32::consts::PI;
use std::thread;

fn generate_symbol() -> Complex<f32> {
    let symbols = [
        Complex::new(PI / 4.0, PI / 4.0),
        Complex::new(-PI / 4.0, PI / 4.0),
        Complex::new(-PI / 4.0, -PI / 4.0),
        Complex::new(PI / 4.0, -PI / 4.0),
    ];
    let mut rng = rand::thread_rng();
    let mut choice = *(symbols.choose(&mut rng).unwrap());
    let normal = Normal::new(0.0, 0.1).unwrap();
    choice.re += normal.sample(&mut rng) as f32;
    choice.im += normal.sample(&mut rng) as f32;
    choice
}

fn main() {
    let handle = thread::spawn(move || {
        let mut figure = Figure::new(10000)
            .xlim([-1.5, 1.5])
            .ylim([-1.5, 1.5])
            .plot_type(PlotType::Dot)
            .color(0x50, 0x20, 0x50);
        Figure::display(&mut figure, |fig| {
            let symbol = generate_symbol();
            fig.plot_complex_stream(&[symbol]);
        });
    });

    handle.join().unwrap();
}
