use crate::utils::{self, Point2D};
use glium::glutin::dpi::LogicalSize;
use glium::implement_vertex;
use glium::Surface;
use itertools_num::linspace;
use num::Complex;

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

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Vertex {
    position: [f32; 2],
}

implement_vertex!(Vertex, position);

impl Vertex {
    fn new(x: f32, y: f32) -> Self {
        Vertex { position: [x, y] }
    }
}

pub struct Renderer<'a> {
    pub events_loop: glium::glutin::EventsLoop,
    display: glium::Display,
    program: glium::Program,
    vertex_buffer: glium::VertexBuffer<Vertex>,
    draw_parameters: glium::DrawParameters<'a>,
}

impl<'a> Renderer<'a> {
    pub fn new() -> Self {
        let events_loop = glium::glutin::EventsLoop::new();
        let context = glium::glutin::ContextBuilder::new().with_vsync(true);
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
            point_size: Some(5.0),
            ..Default::default()
        };

        Renderer {
            events_loop,
            display,
            program,
            vertex_buffer,
            draw_parameters,
        }
    }

    pub fn draw(&mut self, vertices: &[Vertex]) {
        self.vertex_buffer.invalidate();
        let vb = self.vertex_buffer.slice_mut(0..vertices.len()).unwrap();
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

        target.finish().unwrap();
    }
}

#[derive(Default)]
pub struct Figure<'a, T>
where
    T: Into<f32> + Copy,
{
    pub renderer: Option<Renderer<'a>>,
    xlim: Option<[f32; 2]>,
    ylim: Option<[f32; 2]>,
    xlabel: Option<&'a str>,
    ylabel: Option<&'a str>,
    _phantom: std::marker::PhantomData<T>,
}

unsafe impl<'a, T> Send for Figure<'a, T> where T: Into<f32> + Send + Copy {}

impl<'a, T> Figure<'a, T>
where
    T: Into<f32> + Copy + Default,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn init_renderer(mut self) -> Self {
        self.renderer = Some(Renderer::new());
        self
    }

    /// Sets the x min and max limits for plotting.
    pub fn xlim(mut self, xlim: [f32; 2]) -> Self {
        self.xlim = Some(xlim);
        self
    }

    /// Sets the y min and max limits for plotting.
    pub fn ylim(mut self, ylim: [f32; 2]) -> Self {
        self.ylim = Some(ylim);
        self
    }

    pub fn xlabel(mut self, xlabel: &'a str) -> Self {
        self.xlabel = Some(xlabel);
        self
    }

    pub fn ylabel(mut self, ylabel: &'a str) -> Self {
        self.ylabel = Some(ylabel);
        self
    }

    fn normalize(&self, points: &[Point2D]) -> Vec<Vertex>
    where
        T: Into<f32> + Copy,
    {
        let [min_x, max_x] = match self.xlim {
            Some(lim) => lim,
            None => utils::calc_xlims(points),
        };
        let [min_y, max_y] = match self.ylim {
            Some(lim) => lim,
            None => utils::calc_ylims(points),
        };
        let mut vertices = vec![];
        for point in points {
            let x = if max_x != min_x {
                1.5 * (point.x - min_x) / (max_x - min_x) - 0.75
            } else {
                1.5 * point.x - 0.75
            };
            let y = if max_y != min_y {
                1.5 * (point.y - min_y) / (max_y - min_y) - 0.75
            } else {
                1.5 * point.y - 0.75
            };
            vertices.push(Vertex::new(x, y));
        }
        vertices
    }

    pub fn plot(&mut self, points: &[Point2D]) {
        let vertices = self.normalize(&points);
        match self.renderer {
            Some(ref mut render) => render.draw(&vertices),
            None => panic!("Uninitialized renderer for figure"),
        }
    }

    /// Take an array of points and draw it to the screen.
    pub fn plot_xy(&mut self, points: &[(T, T)])
    where
        T: Into<f32> + Copy,
    {
        let points: Vec<Point2D> = points.iter().map(|pt| pt.into()).collect();
        self.plot(&points);
    }

    /// Take an array of y coordinates, interpolate the x, and then plot.
    pub fn plot_y(&mut self, y_coords: &[T])
    where
        T: Into<f32> + Copy,
    {
        let x_coords = linspace(-0.5f32, 0.5f32, y_coords.len());
        let points: Vec<Point2D> = x_coords
            .zip(y_coords.iter())
            .map(|(x, y)| Point2D::new(x, (*y).into()))
            .collect();
        self.plot(&points);
    }

    pub fn plot_complex(&mut self, coords: &[Complex<T>])
    where
        T: Into<f32> + Copy,
    {
        let points: Vec<Point2D> =
            coords.iter().map(|pt| (pt.re, pt.im).into()).collect();
        self.plot(&points);
    }
}
