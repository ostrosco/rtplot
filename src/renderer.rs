use crate::figure::FigureConfig;
use glium::glutin::dpi::LogicalSize;
use glium::{self, implement_vertex, Surface};
use glium_text_rusttype as glium_text;

pub static VERTEX_SHADER: &'static str = r#"
    #version 140

    in vec2 position;
    in vec2 tex_coords;

    out vec2 v_tex_coords;

    void main() {
        gl_Position = vec4(position, 0.0, 1.0);
        v_tex_coords = tex_coords;
    }
"#;

pub static FRAGMENT_SHADER: &'static str = r#"
    #version 140

    in vec2 vec_tex_coords;
    out vec4 color;

    void main() {
        color = vec4(1.0, 0.0, 0.0, 1.0);
    }
"#;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

implement_vertex!(Vertex, position, tex_coords);

impl Vertex {
    pub fn new(x: f32, y: f32) -> Self {
        Vertex {
            position: [x, y],
            tex_coords: [0.0, 0.0],
        }
    }
}

pub struct Renderer<'a> {
    pub events_loop: glium::glutin::EventsLoop,
    display: glium::Display,
    program: glium::Program,
    vertex_buffer: glium::VertexBuffer<Vertex>,
    draw_parameters: glium::DrawParameters<'a>,
    text_system: glium_text::TextSystem,
    font: glium_text::FontTexture,
}
impl<'a> Default for Renderer<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Renderer<'a> {
    pub fn new() -> Self {
        let events_loop = glium::glutin::EventsLoop::new();
        let context = glium::glutin::ContextBuilder::new()
            .with_vsync(true)
            .with_double_buffer(Some(true))
            .with_multisampling(16);
        let window = glium::glutin::WindowBuilder::new()
            .with_dimensions(LogicalSize {
                width: 400.0,
                height: 400.0,
            })
            .with_decorations(true)
            .with_title("Plot");

        let display =
            glium::Display::new(window, context, &events_loop).unwrap();
        let program = glium::Program::from_source(
            &display,
            VERTEX_SHADER,
            FRAGMENT_SHADER,
            None,
        )
        .unwrap();
        let vertex_buffer =
            glium::VertexBuffer::empty_dynamic(&display, 10000).unwrap();
        let draw_parameters = glium::DrawParameters {
            point_size: Some(1.0),
            ..Default::default()
        };
        let text_system = glium_text::TextSystem::new(&display);
        let font = glium_text::FontTexture::new(
            &display,
            ttf_noto_sans::REGULAR,
            128,
            glium_text::FontTexture::ascii_character_list(),
        )
        .unwrap();

        Renderer {
            events_loop,
            display,
            program,
            vertex_buffer,
            draw_parameters,
            text_system,
            font,
        }
    }

    pub fn draw(&mut self, vertices: &[Vertex], config: &FigureConfig) {
        self.vertex_buffer.invalidate();
        let vb = match self.vertex_buffer.slice_mut(0..vertices.len()) {
            Some(slice) => slice,
            None => return,
        };
        vb.write(&vertices);
        let indices =
            glium::index::NoIndices(glium::index::PrimitiveType::Points);

        let mut target = self.display.draw();
        target.clear_color(0.8, 0.8, 0.8, 1.0);
        let vb = self.vertex_buffer.slice(0..vertices.len()).unwrap();
        target
            .draw(
                vb,
                &indices,
                &self.program,
                &glium::uniforms::EmptyUniforms,
                &self.draw_parameters,
            )
            .unwrap();
        self.draw_text(&mut target, config);

        target.finish().unwrap();
    }

    pub fn draw_text<S>(&mut self, target: &mut S, config: &FigureConfig)
    where
        S: glium::Surface,
    {
        if let Some(text) = config.xlabel {
            let label = glium_text::TextDisplay::new(
                &self.text_system,
                &self.font,
                text,
            );
            let text_width = label.get_width() * 0.1;
            let matrix = cgmath::Matrix4::new(
                0.1,
                0.0,
                0.0,
                0.0,
                0.0,
                0.1,
                0.0,
                0.0,
                0.0,
                0.0,
                0.1,
                0.0,
                -text_width / 2.0,
                -0.85,
                0.0,
                1.0,
            );
            glium_text::draw(
                &label,
                &self.text_system,
                target,
                matrix,
                (0.0, 0.0, 0.0, 1.0),
            )
            .unwrap();
        }
        if let Some(text) = config.ylabel {
            let label = glium_text::TextDisplay::new(
                &self.text_system,
                &self.font,
                text,
            );
            let text_width = label.get_width() * 0.1;
            let matrix = cgmath::Matrix4::new(
                0.1,
                0.0,
                0.0,
                0.0,
                0.0,
                0.1,
                0.0,
                0.0,
                0.0,
                1.0,
                0.1,
                0.0,
                -0.85,
                -text_width / 2.0,
                0.0,
                1.0,
            ) * cgmath::Matrix4::from_angle_z(cgmath::Deg(90.0));
            glium_text::draw(
                &label,
                &self.text_system,
                target,
                matrix,
                (0.0, 0.0, 0.0, 1.0),
            )
            .unwrap();
        }
    }
}
