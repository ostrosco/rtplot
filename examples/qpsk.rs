use num::Complex;
use rand::distributions::{Distribution, Normal};
use rand::seq::SliceRandom;
use rtplot::figure::{Figure, PlotType};
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
    let normal = Normal::new(0.0, 0.1);
    choice.re += normal.sample(&mut rng) as f32;
    choice.im += normal.sample(&mut rng) as f32;
    choice
}

fn main() {
    let mut status = true;
    let handle = thread::spawn(move || {
        let mut figure = Figure::new()
            .init_renderer(10000)
            .xlim([-1.0, 1.0])
            .ylim([-1.0, 1.0])
            .plot_type(PlotType::Dot)
            .color(0x50, 0x20, 0x50);
        loop {
            if !status {
                break;
            }

            let symbol = generate_symbol();
            figure.plot_complex_samples(&[symbol]);

            let events_loop = match figure.renderer {
                Some(ref mut rend) => &mut rend.events_loop,
                None => panic!("uninitialized renderer"),
            };
            events_loop.poll_events(|event| {
                use glium::glutin::{Event, WindowEvent};
                #[allow(clippy::single_match)]
                match event {
                    Event::WindowEvent { event, .. } => match event {
                        WindowEvent::Destroyed => status = false,
                        WindowEvent::KeyboardInput {
                            input:
                                glium::glutin::KeyboardInput {
                                    virtual_keycode:
                                        Some(glium::glutin::VirtualKeyCode::Escape),
                                    ..
                                },
                            ..
                        } => status = false,
                        _ => (),
                    },
                    _ => (),
                }
            });
        }
    });

    handle.join().unwrap();
}
