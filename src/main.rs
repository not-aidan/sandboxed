use std::time::Instant;

use nalgebra::Vector2;
use rand::Rng;
use wgpu_text::section::{HorizontalAlign, Layout, Section, Text};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use worm::Worm;

mod base_renderer;
mod renderer;
mod sprite;
mod world;
mod worm;

use self::{
    renderer::Renderer,
    world::{Coordinate, World, WORLD_SIZE},
};

const WORLD_UPDATE_TIME: f32 = 0.1;
const TARGET_FPS: f64 = 1.0 / 60.0;

#[tokio::main]
async fn main() -> Result<(), ()> {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let mut last_frame = Instant::now();
    let mut last_world_step = Instant::now();
    let mut world = World::default();
    let mut renderer = Renderer::new(window).await;

    let mut worms = vec![Worm::new(
        7,
        Vector2::new(30.0, 30.0),
        Vector2::new(1.0, 1.0).normalize(),
        10.0,
        4.0,
    )];

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == renderer.window().id() => match event {
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                } => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(size) => {
                    renderer.resize(*size);
                }
                _ => {}
            },
            Event::RedrawRequested(window_id) if window_id == renderer.window().id() => {
                let time = Instant::now();

                if time.duration_since(last_frame).as_secs_f64() < TARGET_FPS {
                    return;
                }

                let time_since_world_step = time.duration_since(last_world_step).as_secs_f32();
                if time_since_world_step >= WORLD_UPDATE_TIME {
                    let coordinate = Coordinate::new(WORLD_SIZE / 2, WORLD_SIZE - 1);

                    if world.get_cell(&coordinate) == Some(world::CellElement::Air) {
                        world.set_cell(
                            &coordinate,
                            world::CellElement::Sand(Vector2::new(
                                0.0,
                                rand::thread_rng().gen_range(-2.0..=0.0),
                            )),
                        );
                    }

                    let mut forces = Vec::<world::Force>::new();
                    for worm in worms.iter_mut() {
                        worm.step_ai(time_since_world_step);
                        for segment in worm.segments.iter() {
                            forces.push(segment.force());
                        }
                    }

                    world.update(&forces);
                    last_world_step = time;
                }

                const PRECISION: f32 = 10.0;

                let fps = (1.0 / time.duration_since(last_frame).as_secs_f32())
                    .round()
                    .to_string()
                    + " FPS";

                // text
                let section = Section::default()
                    .add_text(Text::new(&fps))
                    .with_layout(Layout::default().h_align(HorizontalAlign::Left));

                match renderer.render(&world, &worms, &[section]) {
                    Ok(_) => {}
                    // Reconfigure the surface if lost
                    Err(wgpu::SurfaceError::Lost) => renderer.resize(renderer.size()),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }

                last_frame = time;
            }
            Event::MainEventsCleared => {
                renderer.window().request_redraw();
            }
            _ => {}
        }
    });
}
