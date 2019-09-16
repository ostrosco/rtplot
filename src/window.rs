use crate::figure::{FigureConfig, PlotType};
use glium::glutin::dpi::LogicalSize;
use glium::uniform;
use glium::{self, implement_vertex, Surface};
use glium_text_rusttype as glium_text;
use itertools_num::linspace;
use lyon::math::{point, Point};
use lyon::tessellation::basic_shapes::{
    fill_circle, fill_polyline, stroke_polyline, stroke_quad,
};
use lyon::tessellation::geometry_builder::{
    BuffersBuilder, VertexBuffers, VertexConstructor,
};
use lyon::tessellation::*;
use lyon::tessellation::{FillOptions, StrokeOptions};

pub static VERTEX_SHADER: &str = r#"
    #version 140
    in vec3 position;
    in vec3 rgb;
    out vec3 rgb_frag;
    uniform mat4 projection;
    void main() {
        gl_Position = projection * vec4(position, 1.0);
        rgb_frag = rgb;
    }
"#;

pub static FRAGMENT_SHADER: &str = r#"
    #version 140
    in vec3 rgb_frag;
    out vec4 color;
    void main() {
        color = vec4(rgb_frag, 1.0);
    }
"#;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Vertex {
    position: [f32; 3],
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
            position: [x, y, 0.0],
            rgb,
        }
    }
}

enum ZDepth {
    Near,
    Far,
}

struct VertexCtor([u8; 3], ZDepth);
impl VertexConstructor<lyon::tessellation::StrokeVertex, Vertex>
    for VertexCtor
{
    fn new_vertex(
        &mut self,
        vertex: lyon::tessellation::StrokeVertex,
    ) -> Vertex {
        let rgb: [f32; 3] = [
            f32::from(self.0[0]) / 255.0,
            f32::from(self.0[1]) / 255.0,
            f32::from(self.0[2]) / 255.0,
        ];
        let pos = vertex.position.to_array();
        let position = match self.1 {
            ZDepth::Far => [pos[0], pos[1], 0.0],
            ZDepth::Near => [pos[0], pos[1], 1.0],
        };
        Vertex { position, rgb }
    }
}

impl VertexConstructor<lyon::tessellation::FillVertex, Vertex> for VertexCtor {
    fn new_vertex(&mut self, vertex: lyon::tessellation::FillVertex) -> Vertex {
        let rgb: [f32; 3] = [
            f32::from(self.0[0]) / 255.0,
            f32::from(self.0[1]) / 255.0,
            f32::from(self.0[2]) / 255.0,
        ];
        let pos = vertex.position.to_array();
        let position = match self.1 {
            ZDepth::Far => [pos[0], pos[1], 0.0],
            ZDepth::Near => [pos[0], pos[1], 1.0],
        };
        Vertex { position, rgb }
    }
}

pub struct Window<'a> {
    pub events_loop: glium::glutin::EventsLoop,
    display: glium::Display,
    program: glium::Program,
    draw_parameters: glium::DrawParameters<'a>,
    text_system: glium_text::TextSystem,
    font: glium_text::FontTexture,
}

impl<'a> Default for Window<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Window<'a> {
    pub fn new() -> Self {
        let events_loop = glium::glutin::EventsLoop::new();
        let context = glium::glutin::ContextBuilder::new()
            .with_vsync(true)
            .with_double_buffer(Some(true))
            .with_depth_buffer(24)
            .with_multisampling(2);
        let window = glium::glutin::WindowBuilder::new()
            .with_dimensions(LogicalSize {
                width: 800.0,
                height: 800.0,
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

        let draw_parameters = glium::DrawParameters {
            depth: glium::Depth {
                write: true,
                test: glium::DepthTest::IfLess,
                ..Default::default()
            },
            ..Default::default()
        };

        let text_system = glium_text::TextSystem::new(&display);
        let font = glium_text::FontTexture::new(
            &display,
            ttf_noto_sans::REGULAR,
            70,
            glium_text::FontTexture::ascii_character_list(),
        )
        .unwrap();

        Self {
            events_loop,
            display,
            program,
            draw_parameters,
            text_system,
            font,
        }
    }

    pub fn draw(&mut self, vertices: &[Vertex], config: &FigureConfig) {
        let mut target = self.display.draw();
        let color = (169.0 / 255.0, 169.0 / 255.0, 169.0 / 255.0, 1.0);
        target.clear_color_and_depth(color, 1.0);
        let mut mesh: VertexBuffers<Vertex, u32> = VertexBuffers::new();
        self.draw_text(&mut target, config);
        self.draw_grid(&mut mesh);

        let points: Vec<Point> = vertices
            .iter()
            .map(|x| point(x.position[0], x.position[1]))
            .collect();

        match config.plot_type {
            PlotType::Line => {
                stroke_polyline(
                    points.iter().cloned(),
                    false,
                    &StrokeOptions::tolerance(0.01).with_line_width(0.002),
                    &mut BuffersBuilder::new(
                        &mut mesh,
                        VertexCtor(config.color, ZDepth::Near),
                    ),
                )
                .expect("Could not draw line plot");
            }
            PlotType::Dot => {
                for point in points {
                    fill_circle(
                        point,
                        0.01,
                        &FillOptions::tolerance(0.01),
                        &mut BuffersBuilder::new(
                            &mut mesh,
                            VertexCtor(config.color, ZDepth::Near),
                        ),
                    )
                    .expect("Could not draw dot plot");
                }
            }
        }

        let (w, h) = self.display.get_framebuffer_dimensions();
        let aspect = w as f32 / h as f32;
        let ortho_mat = cgmath::ortho(-aspect, aspect, -1.0, 1.0, -1.0, 1.0);
        let ortho: &[[f32; 4]; 4] = ortho_mat.as_ref();
        let uniforms = uniform! {
            projection: *ortho,
        };

        let vertex_buffer =
            glium::VertexBuffer::new(&self.display, &mesh.vertices)
                .expect("Could not create vertex buffer");
        let indices = glium::IndexBuffer::new(
            &self.display,
            glium::index::PrimitiveType::TrianglesList,
            &mesh.indices,
        )
        .expect("Could not create index buffer");

        target
            .draw(
                &vertex_buffer,
                &indices,
                &self.program,
                &uniforms,
                &self.draw_parameters,
            )
            .expect("Could not draw the frame");

        target.finish().expect("Could not finish the frame");
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
            .expect("Could not draw x label");
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
            .expect("Could not draw y label");
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
                .expect("Could not draw x axis values");
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
                let text_height = tick_str.get_height() * 0.05;
                #[rustfmt::skip]
                let matrix = ortho_mat * cgmath::Matrix4::new(
                    0.05, 0.0, 0.0, 0.0,
                    0.0, 0.05, 0.0, 0.0,
                    0.0, 0.0, 0.05, 0.0,
                    -0.85, coord - text_height / 2.0, 0.0, 1.0,
                );
                glium_text::draw(
                    &tick_str,
                    &self.text_system,
                    target,
                    matrix,
                    (0.0, 0.0, 0.0, 1.0),
                )
                .expect("Could not draw y axis labels");
            }
        }
    }

    fn draw_grid(&mut self, mesh: &mut VertexBuffers<Vertex, u32>) {
        let mut tessellator = FillTessellator::new();

        for tick in linspace(-0.75, 0.75, 6) {
            fill_polyline(
                [
                    point(tick - 0.001, 0.75),
                    point(tick - 0.001, -0.75),
                    point(tick + 0.001, -0.75),
                    point(tick + 0.001, 0.75),
                ]
                .iter()
                .cloned(),
                &mut tessellator,
                &FillOptions::tolerance(0.01),
                &mut BuffersBuilder::new(
                    mesh,
                    VertexCtor([0x5d, 0x5d, 0x5d], ZDepth::Far),
                ),
            )
            .expect("Could not draw grid");
        }

        for tick in linspace(-0.75, 0.75, 5) {
            fill_polyline(
                [
                    point(0.75, tick - 0.001),
                    point(-0.75, tick - 0.001),
                    point(-0.75, tick + 0.001),
                    point(0.75, tick + 0.001),
                ]
                .iter()
                .cloned(),
                &mut tessellator,
                &FillOptions::tolerance(0.01),
                &mut BuffersBuilder::new(
                    mesh,
                    VertexCtor([0x5d, 0x5d, 0x5d], ZDepth::Far),
                ),
            )
            .expect("Could not draw grid");
        }

        stroke_quad(
            point(-0.75, -0.75),
            point(-0.75, 0.75),
            point(0.75, 0.75),
            point(0.75, -0.75),
            &StrokeOptions::tolerance(0.01).with_line_width(0.001),
            &mut BuffersBuilder::new(mesh, VertexCtor([0, 0, 0], ZDepth::Near)),
        )
        .unwrap();
    }
}
