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

#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    position: [f32; 2],
}

implement_vertex!(Vertex, position);

impl Vertex {
    fn new(x: f32, y: f32) -> Self {
        Vertex { position: [x, y] }
    }
}

pub struct Renderer {
    pub events_loop: glium::glutin::EventsLoop,
    display: glium::Display,
    program: glium::Program,
    vertex_buffer: glium::VertexBuffer<Vertex>,
}

impl Renderer {
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
            glium::VertexBuffer::empty_dynamic(&display, 300000).unwrap();

        Renderer {
            events_loop,
            display,
            program,
            vertex_buffer,
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
        target
            .draw(
                &self.vertex_buffer,
                &indices,
                &self.program,
                &glium::uniforms::EmptyUniforms,
                &Default::default(),
            )
            .unwrap();

        target.finish().unwrap();
    }
}

pub struct Figure<T> {
    pub renderer: Option<Renderer>,
    _phantom: std::marker::PhantomData<T>,
}

unsafe impl<T> Send for Figure<T> {}

impl<T> Default for Figure<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Figure<T> {
    pub fn new() -> Self {
        Figure {
            renderer: None,
            _phantom: Default::default(),
        }
    }

    pub fn init_renderer(mut self) -> Self {
        self.renderer = Some(Renderer::new());
        self
    }

    fn normalize(points: &[(T, T)]) -> Vec<Vertex>
    where
        T: Into<f32> + Copy,
    {
        // Grab the min and max points of the data and normalize it to fix onto
        // the screen.
        let points: Vec<(f32, f32)> =
            points.iter().map(|x| (x.0.into(), x.1.into())).collect();
        let min_x = points
            .iter()
            .min_by(|x, y| x.0.partial_cmp(&y.0).unwrap())
            .unwrap()
            .0;
        let max_x = points
            .iter()
            .max_by(|x, y| x.0.partial_cmp(&y.0).unwrap())
            .unwrap()
            .0;
        let min_y = points
            .iter()
            .min_by(|x, y| x.1.partial_cmp(&y.1).unwrap())
            .unwrap()
            .1;
        let max_y = points
            .iter()
            .max_by(|x, y| x.1.partial_cmp(&y.1).unwrap())
            .unwrap()
            .1;

        let mut vertices = vec![];
        for point in points {
            let x = if max_x != min_x {
                2.0 * (point.0 - min_x) / (max_x - min_x) - 1.0
            } else {
                2.0 * point.0 - 1.0
            };
            let y = if max_y != min_y {
                0.8 * (point.1 - min_y) / (max_y - min_y) - 0.4
            } else {
                point.1 - 0.4
            };
            vertices.push(Vertex::new(x, y));
        }
        vertices
    }


    /// Take an array of points and draw it to the screen.
    pub fn plot_xy(&mut self, points: &[(T, T)])
    where
        T: Into<f32> + Copy,
    {
        let vertices = Figure::normalize(&points);
        match self.renderer {
            Some(ref mut render) => render.draw(&vertices),
            None => panic!("Uninitialized renderer for figure"),
        }
    }

    /// Take an array of y coordinates, interpolate the x, and then plot.
    pub fn plot_y(&mut self, y_coords: &[T])
    where
        T: Into<f32> + Copy,
    {
        let x_coords = linspace(-0.5f32, 0.5f32, y_coords.len());
        let points: Vec<(f32, f32)> = x_coords
            .zip(y_coords.iter())
            .map(|(x, y)| (x, (*y).into()))
            .collect();
        let vertices = Figure::normalize(&points);
        match self.renderer {
            Some(ref mut render) => render.draw(&vertices),
            None => panic!("Uninitialized renderer for figure"),
        }
    }

    pub fn plot_complex(&mut self, coords: &[Complex<T>])
    where
        T: Into<f32> + Copy,
    {
        let points: Vec<(T, T)> = coords.iter().map(|x| (x.re, x.im)).collect();
        self.plot_xy(&points);
    }
}
