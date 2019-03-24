#[macro_use]
extern crate glium;

use glium::glutin::dpi::LogicalSize;
use glium::Surface;
use itertools_num::linspace;
use std::f32::consts::PI;

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
}

implement_vertex!(Vertex, position);

fn calculate_sin(phase: f32) -> Vec<Vertex> {
    let mut sin_vals: Vec<_> = linspace(0.0, 100.0, 10000)
        .map(|x| Vertex {
            position: [x as f32, (PI / 8.0 * x as f32 + phase).sin()],
        })
        .collect();

    let min_x = sin_vals
        .iter()
        .min_by(|x, y| x.position[0].partial_cmp(&y.position[0]).unwrap())
        .unwrap()
        .position[0];
    let max_x = sin_vals
        .iter()
        .max_by(|x, y| x.position[0].partial_cmp(&y.position[0]).unwrap())
        .unwrap()
        .position[0];
    let min_y = sin_vals
        .iter()
        .min_by(|x, y| x.position[1].partial_cmp(&y.position[1]).unwrap())
        .unwrap()
        .position[1];
    let max_y = sin_vals
        .iter()
        .max_by(|x, y| x.position[1].partial_cmp(&y.position[1]).unwrap())
        .unwrap()
        .position[1];

    for sin_val in sin_vals.iter_mut() {
        sin_val.position[0] =
            2.0 * (sin_val.position[0] - min_x) / (max_x - min_x) - 1.0;
        sin_val.position[1] =
            (sin_val.position[1] - min_y) / (max_y - min_y) - 0.5;
    }
    sin_vals
}

fn main() {
    let mut phase = 0.0;


    let mut events_loop = glium::glutin::EventsLoop::new();
    let context = glium::glutin::ContextBuilder::new().with_vsync(true);
    let window = glium::glutin::WindowBuilder::new()
        .with_dimensions(LogicalSize {
            width: 400.0,
            height: 400.0,
        })
        .with_decorations(true)
        .with_title("lyon + glium basic example");
    let display = glium::Display::new(window, context, &events_loop).unwrap();
    let program = glium::Program::from_source(
        &display,
        VERTEX_SHADER,
        FRAGMENT_SHADER,
        None,
    )
    .unwrap();


    let mut status = true;
    loop {
        if !status {
            break;
        }

        let sin_vals = calculate_sin(phase);
        let vertex_buffer = glium::VertexBuffer::new(&display, &sin_vals).unwrap();
        let indices = glium::index::NoIndices(glium::index::PrimitiveType::Points);
        phase += PI / 16.0;

        let mut target = display.draw();
        target.clear_color(0.8, 0.8, 0.8, 1.0);
        target
            .draw(
                &vertex_buffer,
                &indices,
                &program,
                &glium::uniforms::EmptyUniforms,
                &Default::default(),
            )
            .unwrap();

        target.finish().unwrap();

        events_loop.poll_events(|event| {
            use glium::glutin::{Event, WindowEvent};
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

pub static VERTEX_SHADER: &'static str = r#"
    #version 140

    in vec2 position;

    void main() {
        gl_Position = vec4(position, 0.0, 1.0);
    }
"#;

pub static FRAGMENT_SHADER: &'static str = r#"
    #version 140

    out vec4 color;

    void main() {
        color = vec4(1.0, 0.0, 0.0, 1.0);
    }
"#;
