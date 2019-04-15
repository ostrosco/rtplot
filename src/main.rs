use itertools_num::linspace;
use rtplot::figure::Figure;
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
    let mut status = true;
    let handle = thread::spawn(move || {
        let mut figure = Figure::new()
            .init_renderer(10000)
            .xlim([0.0, 1.0])
            .ylim([-10.0, 10.0])
            .xlabel("Time (s)")
            .ylabel("Amplitude")
            .color(0x80, 0x00, 0x80);
        loop {
            if !status {
                break;
            }

            let sin_vals = calculate_sin(phase);
            figure.plot_y(&sin_vals);
            phase += PI / 20.0;

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
