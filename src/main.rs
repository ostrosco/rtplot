use itertools_num::linspace;
use rtplot::figure::Figure;
use std::f32::consts::PI;

fn calculate_sin(phase: f32) -> Vec<(f32, f32)> {
    let sin_vals: Vec<_> = linspace(0.0, 100.0, 10000)
        .map(|x| (x as f32, (PI / 8.0 * x as f32 + phase).sin()))
        .collect();

    sin_vals
}

fn main() {
    let mut phase = 0.0;

    let mut figure = Figure::new();

    let mut status = true;
    loop {
        if !status {
            break;
        }

        let sin_vals = calculate_sin(phase);
        figure.plot(&sin_vals);
        phase += PI / 20.0;

        figure.events_loop.poll_events(|event| {
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
}
