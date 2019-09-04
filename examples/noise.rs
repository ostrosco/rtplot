use rtplot::figure::{Figure, PlotType};
use std::thread;
use rand::distributions::{Distribution, Normal};

fn main() {
    let mut status = true;
    let handle = thread::spawn(move || {
        let normal = Normal::new(0.0, 1.0);
        let mut rng = rand::thread_rng();
        let mut figure = Figure::new()
            .init_renderer(100)
            .ylim([-1.0, 1.0])
            .xlabel("Time (s)")
            .ylabel("Amplitude")
            .plot_type(PlotType::Line)
            .color(0x80, 0x00, 0x80);
        loop {
            if !status {
                break;
            }

            let v: Vec<f32> = normal.sample_iter(&mut rng).take(10).map(|x| x as f32).collect();
            figure.plot_samples(&v);

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
