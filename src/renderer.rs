use crate::figure::{FigureConfig, PlotType};
use glium::glutin::dpi::LogicalSize;
use glium::uniform;
use glium::{self, implement_vertex, Surface};
use imgui::{Context, FontConfig, FontGlyphRanges, FontSource, Ui};
use imgui_glium_renderer::Renderer;
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use itertools_num::linspace;

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

pub struct WindowState {
    pub events_loop: glium::glutin::EventsLoop,
    pub display: glium::Display,
    pub imgui: Context,
    pub platform: WinitPlatform,
    pub renderer: Renderer,
    font_size: f32,
}

impl WindowState {
    pub fn new(num_points: usize) -> Self {
        let events_loop = glium::glutin::EventsLoop::new();
        let context = glium::glutin::ContextBuilder::new()
            .with_vsync(true)
            .with_double_buffer(Some(true))
            .with_multisampling(2);
        let builder = glium::glutin::WindowBuilder::new()
            .with_dimensions(LogicalSize {
                width: 640.0,
                height: 480.0,
            })
            .with_decorations(true)
            .with_title("Plot");

        let display =
            glium::Display::new(builder, context, &events_loop).unwrap();

        let mut imgui = Context::create();
        imgui.set_ini_filename(None);

        let mut platform = WinitPlatform::init(&mut imgui);
        {
            let gl_window = display.gl_window();
            let window = gl_window.window();
            platform.attach_window(imgui.io_mut(), &window, HiDpiMode::Rounded);
        }

        let hidpi_factor = platform.hidpi_factor();
        let font_size = (13.0 * hidpi_factor) as f32;
        imgui.fonts().add_font(&[
            FontSource::DefaultFontData {
                config: Some(FontConfig {
                    size_pixels: font_size,
                    ..FontConfig::default()
                }),
            },
            FontSource::TtfData {
                data: ttf_noto_sans::REGULAR,
                size_pixels: font_size,
                config: Some(FontConfig {
                    rasterizer_multiply: 1.75,
                    ..FontConfig::default()
                }),
            },
        ]);

        imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;
        let renderer = Renderer::init(&mut imgui, &display)
            .expect("Failed to initialized renderer");

        Self {
            events_loop,
            display,
            imgui,
            platform,
            renderer,
            font_size,
        }
    }

    pub fn draw(&mut self, vertices: &[Vertex], config: &FigureConfig) {
        /*
        let (w, h) = self.display.get_framebuffer_dimensions();
        let aspect = w as f32 / h as f32;
        let ortho_mat = cgmath::ortho(-aspect, aspect, -1.0, 1.0, -1.0, 1.0);
        let ortho: &[[f32; 4]; 4] = ortho_mat.as_ref();
        let uniforms = uniform! {
            projection: *ortho,
        };
        self.vertex_buffer.invalidate();

        let mut target = self.display.draw();
        target.clear_color(0.8, 0.8, 0.8, 1.0);
        // If there are no vertices provided during this draw, draw the axes
        // and stop.
        if vertices.is_empty() {
            target.finish().unwrap();
            return;
        }
        let vb = match self.vertex_buffer.slice_mut(0..vertices.len()) {
            Some(slice) => slice,
            None => {
                target.finish().unwrap();
                return;
            }
        };
        vb.write(&vertices);
        let plot_type = match config.plot_type {
            PlotType::Dot => glium::index::PrimitiveType::Points,
            PlotType::Line => glium::index::PrimitiveType::LineStrip,
        };
        let indices = glium::index::NoIndices(plot_type);

        let vb = match self.vertex_buffer.slice(0..vertices.len()) {
            Some(slice) => slice,
            None => {

                target.finish().unwrap();
                return;
            }
        };
        target.draw(
                vb,
                &indices,
                &self.program,
                &uniforms,
                &self.draw_parameters,
            )
            .unwrap();

        target.finish().unwrap();
        */
    }
}
