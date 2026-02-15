use std::collections::HashMap;
use std::time::Instant;
use pixels::{Pixels, SurfaceTexture};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{EventLoop, ActiveEventLoop, ControlFlow};
use winit::window::{Window, WindowId};

const WIDTH: u32 = 1080;
const HEIGHT: u32 = 720;


struct RenderContext {
    window: &'static Window,
    pixels: Pixels<'static>,
}

impl RenderContext {
    fn new(window: &'static Window) -> Self {
        let size = window.inner_size();
        let surface_texture = SurfaceTexture::new(size.width, size.height, window);
        let pixels = Pixels::new(WIDTH, HEIGHT, surface_texture).unwrap();

        Self { window, pixels }
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.pixels.resize_surface(width, height).unwrap();
    }
}

struct Game {
    target_window: Option<WindowId>,
    frame_count: usize,
    last_frame_time: Instant,
}

impl Game {
    fn new() -> Self {
        Self {
            target_window: None,
            last_frame_time: Instant::now(),
            frame_count: 0,
        }
    }

    fn draw(&mut self, context: &mut RenderContext) {
        let now = Instant::now();
        let delta_time = now.duration_since(self.last_frame_time).as_secs_f32();

        self.last_frame_time = now;

        if delta_time > 0.0 {
            let fps = 1.0 / delta_time;
            let title = format!("FPS: {:.2}", fps);
            if self.frame_count % 30 == 0 {
                context.window.set_title(&title);
            }
        }

        self.frame_count += 1;
        let frame = context.pixels.frame_mut();

        let softness = 3.0;

        let time = self.frame_count as f32 * 0.05;
        let wave = time.sin();
        let dynamic_radius = 100.0 + (wave * 20.0);

        let center_x = (WIDTH / 2) as f32;
        let center_y = (HEIGHT / 2) as f32;
        let color_mod = (self.frame_count % 255) as u8; 

        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let x = (i % WIDTH as usize * 255 / WIDTH as usize) as u8;
            let y = (i / WIDTH as usize * 255 / WIDTH as usize) as u8;

            let dx = (i % WIDTH as usize) as f32 - center_x;
            let dy = (i / WIDTH as usize) as f32 - center_y;

            // sqrt (x1 - x2)^2 + (y1 - y2)^2 
            let distance = (dx*dx + dy*dy).sqrt();

            let radius = dynamic_radius;

            let local_pulse = (time + (x as f32 * 0.1)).sin() * 10.0;
            let local_radius = 100.0 + local_pulse;

            let alpha = ((local_radius - distance) / softness).clamp(0.0, 1.0);
            let r = (255.0 * alpha + 24 as f32 * (1.0 - alpha)) as u8;
            let g = (255.0 * alpha + 24 as f32 * (1.0 - alpha)) as u8;
            let b = (255.0 * alpha + 24 as f32 * (1.0 - alpha)) as u8;

            pixel.copy_from_slice(&[r, g, b, 255]);
        }
    }
}

#[derive(Default)]
struct App {
    resources: HashMap<WindowId, RenderContext>,
    game: Option<Game>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        event_loop.set_control_flow(ControlFlow::Poll);

        if self.game.is_none() {
            self.game = Some(Game::new());
        }

        if self.resources.is_empty() {
            let window_attributes = Window::default_attributes()
                                        .with_title("A Game Window")
                                        .with_inner_size(winit::dpi::LogicalSize::new(WIDTH * 2, HEIGHT * 2));

            let window = event_loop.create_window(window_attributes).unwrap();

            let window: &'static Window = Box::leak(Box::new(window));
            let window_id = window.id();

            let context = RenderContext::new(window);
            self.resources.insert(window_id, context);

            if let Some(game) = self.game.as_mut() {
                game.target_window = Some(window_id);
            }
            println!("Created Window with ID: {:?}", window_id);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id:WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("Closing Window: {:?}", window_id);
                self.resources.remove(&window_id);

                if self.resources.is_empty() {
                    event_loop.exit();
                }
            }
            WindowEvent::Resized(size) => {
                if let Some(context) = self.resources.get_mut(&window_id) {
                    context.resize(size.width, size.height);
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(game) = self.game.as_mut() {
                    if game.target_window == Some(window_id) {
                        if let Some(context) = self.resources.get_mut(&window_id) {
                            game.draw(context);
                            if context.pixels.render().is_err() {
                                event_loop.exit();
                            }
                            context.window.request_redraw();
                        }
                    }
                }
            }
            _ => (),
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let mut app = App::default();
    event_loop.run_app(&mut app).unwrap();
}
