use crate::figure::FigureConfig;
use glium::uniform;
use glium::glutin::dpi::LogicalSize;
use glium::{self, implement_vertex, Surface};
use glium_text_rusttype as glium_text;
use itertools_num::linspace;

pub static VERTEX_SHADER: &'static str = r#"
    #version 140

    in vec2 position;
    in vec3 rgb;
    out vec3 rgb_frag;
    uniform mat4 projection;

    void main() {
        gl_Position = projection * vec4(position, 0.0, 1.0);
        rgb_frag = rgb;
    }
"#;

pub static FRAGMENT_SHADER: &'static str = r#"
    #version 140

    in vec3 rgb_frag;
    out vec4 color;

    void main() {
        color = vec4(rgb_frag, 1.0);
    }
"#;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Vertex {
    position: [f32; 2],
    rgb: [f32; 3],
}

implement_vertex!(Vertex, position, rgb);

impl Vertex {
    pub fn new(x: f32, y: f32, rgb: [u8; 3]) -> Self {
        let rgb: [f32; 3] = [
            f32::from(rgb[0]) / 255.0,
            f32::from(rgb[1]) / 255.0,
            f32::from(rgb[2]) / 255.0,
        ];
        Vertex {
            position: [x, y],
            rgb,
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
    bounding_box: Vec<Vertex>,
}

impl<'a> Renderer<'a> {
    pub fn new(num_points: usize) -> Self {
        let events_loop = glium::glutin::EventsLoop::new();
        let context = glium::glutin::ContextBuilder::new()
            .with_vsync(true)
            .with_double_buffer(Some(true))
            .with_multisampling(2);
        let window = glium::glutin::WindowBuilder::new()
            .with_dimensions(LogicalSize {
                width: 640.0,
                height: 480.0,
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
            glium::VertexBuffer::empty_dynamic(&display, num_points).unwrap();
        let draw_parameters = glium::DrawParameters {
            point_size: Some(2.0),
            ..Default::default()
        };
        let text_system = glium_text::TextSystem::new(&display);
        let font = glium_text::FontTexture::new(
            &display,
            ttf_noto_sans::REGULAR,
            64,
            glium_text::FontTexture::ascii_character_list(),
        )
        .unwrap();

        let mut bounding_box = vec![
            Vertex::new(-0.75, -0.75, [0, 0, 0]),
            Vertex::new(-0.75, 0.75, [0, 0, 0]),
            Vertex::new(-0.75, 0.75, [0, 0, 0]),
            Vertex::new(0.75, 0.75, [0, 0, 0]),
            Vertex::new(0.75, 0.75, [0, 0, 0]),
            Vertex::new(0.75, -0.75, [0, 0, 0]),
            Vertex::new(0.75, -0.75, [0, 0, 0]),
            Vertex::new(-0.75, -0.75, [0, 0, 0]),
        ];
        for tick in linspace(-0.75, 0.75, 6) {
            bounding_box.push(Vertex::new(tick, -0.70, [0, 0, 0]));
            bounding_box.push(Vertex::new(tick, -0.75, [0, 0, 0]));
        }

        for tick in linspace(-0.75, 0.75, 5) {
            bounding_box.push(Vertex::new(-0.70, tick, [0, 0, 0]));
            bounding_box.push(Vertex::new(-0.75, tick, [0, 0, 0]));
        }

        Renderer {
            events_loop,
            display,
            program,
            vertex_buffer,
            draw_parameters,
            text_system,
            font,
            bounding_box,
        }
    }

    pub fn draw(&mut self, vertices: &[Vertex], config: &FigureConfig) {
        let (w, h) = self.display.get_framebuffer_dimensions();
        let aspect = w as f32 / h as f32;
        let ortho_mat = cgmath::ortho(-aspect, aspect, -1.0, 1.0, -1.0, 1.0);
        let ortho: &[[f32; 4]; 4] = ortho_mat.as_ref();
        let uniforms = uniform! {
            projection: *ortho,
        };
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
                &uniforms,
                &self.draw_parameters,
            )
            .unwrap();
        self.draw_axis(&mut target);
        self.draw_text(&mut target, config);

        target.finish().unwrap();
    }

    pub fn draw_text<S>(&mut self, target: &mut S, config: &FigureConfig)
    where
        S: glium::Surface,
    {
        let (w, h) = self.display.get_framebuffer_dimensions();
        let aspect = w as f32 / h as f32;
        let ortho_mat = cgmath::ortho(-aspect, aspect, -1.0, 1.0, -1.0, 1.0);
        if let Some(text) = config.xlabel {
            let label = glium_text::TextDisplay::new(
                &self.text_system,
                &self.font,
                text,
            );
            let text_width = label.get_width() * 0.1;
            #[rustfmt::skip]
            let matrix = ortho_mat * cgmath::Matrix4::new(
                0.1, 0.0, 0.0, 0.0,
                0.0, 0.1, 0.0, 0.0,
                0.0, 0.0, 0.1, 0.0,
                -text_width / 2.0, -0.90, 0.0, 1.0,
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
            #[rustfmt::skip]
            let matrix = ortho_mat * cgmath::Matrix4::new(
                0.1, 0.0, 0.0, 0.0,
                0.0, 0.1, 0.0, 0.0,
                0.0, 1.0, 0.1, 0.0,
                -0.90, -text_width / 2.0, 0.0, 1.0,
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
        if let Some([xmin, xmax]) = config.xlim {
            for (coord, tick) in
                linspace(-0.75, 0.75, 6).zip(linspace(xmin, xmax, 6))
            {
                let tick_str = glium_text::TextDisplay::new(
                    &self.text_system,
                    &self.font,
                    &format!("{:.02}", tick),
                );
                let text_width = tick_str.get_width() * 0.05;
                #[rustfmt::skip]
                let matrix = ortho_mat * cgmath::Matrix4::new(
                    0.05, 0.0, 0.0, 0.0,
                    0.0, 0.05, 0.0, 0.0,
                    0.0, 0.0, 0.05, 0.0,
                    coord - text_width / 2.0, -0.80, 0.0, 1.0,
                );
                glium_text::draw(
                    &tick_str,
                    &self.text_system,
                    target,
                    matrix,
                    (0.0, 0.0, 0.0, 1.0),
                )
                .unwrap();
            }
        }
        if let Some([ymin, ymax]) = config.ylim {
            for (coord, tick) in
                linspace(-0.75, 0.75, 5).zip(linspace(ymin, ymax, 5))
            {
                let tick_str = glium_text::TextDisplay::new(
                    &self.text_system,
                    &self.font,
                    &format!("{:.02}", tick),
                );
                #[rustfmt::skip]
                let matrix = ortho_mat * cgmath::Matrix4::new(
                    0.05, 0.0, 0.0, 0.0,
                    0.0, 0.05, 0.0, 0.0,
                    0.0, 0.0, 0.05, 0.0,
                    -0.85, coord, 0.0, 1.0,
                );
                glium_text::draw(
                    &tick_str,
                    &self.text_system,
                    target,
                    matrix,
                    (0.0, 0.0, 0.0, 1.0),
                )
                .unwrap();
            }
        }
    }

    pub fn draw_axis<S>(&mut self, target: &mut S)
    where
        S: glium::Surface,
    {
        self.vertex_buffer.invalidate();
        let vb = match self.vertex_buffer.slice_mut(0..30) {
            Some(slice) => slice,
            None => return,
        };
        vb.write(&self.bounding_box);
        let indices =
            glium::index::NoIndices(glium::index::PrimitiveType::LinesList);
        let vb = self.vertex_buffer.slice(0..30).unwrap();
        target
            .draw(
                vb,
                &indices,
                &self.program,
                &glium::uniforms::EmptyUniforms,
                &self.draw_parameters,
            )
            .unwrap();
    }
}
