use std::time::SystemTime;

use engine::Engine;
use scene::create_minecraft_scene;
use windows::Win32::Foundation::*;
use winit::dpi::LogicalSize;
use winit::event::VirtualKeyCode;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

use winit::platform::windows::WindowExtWindows;

pub mod render_backend;

mod simple_triangle;
use simple_triangle::SimpleTriangleScene;

mod camera;
mod engine;
mod object;
mod scene;
mod simple_gbuffer_pass;
mod vertex_colour_stage;

fn main() {
    let mut input = WinitInputHelper::new();

    let event_loop = EventLoop::new();

    let width = 800.;
    let height = 600.;
    let _window = WindowBuilder::new()
        .with_title("D3D11")
        .with_inner_size(LogicalSize::new(width, height))
        .build(&event_loop)
        .unwrap();

    let hwnd = _window.hwnd();

    //let triangle_scene = SimpleTriangleScene::new(hwnd as HWND);

    let mut time = 0usize;

    let mut last_time = SystemTime::now();

    let mut engine = Engine::new(hwnd as HWND)
        .add_basic_renderer(800, 600)
        .add_scene(&create_minecraft_scene);

    event_loop.run(move |event, _, control_flow| {
        // Pass every event to the WindowInputHelper.
        // It will return true when the last event has been processed and it is time to run your application logic.

        let now = SystemTime::now();
        let delta_time = now.duration_since(last_time).expect("get delta time");
        let delta_time = delta_time.subsec_millis();

        time += delta_time as usize;

        if input.update(&event) {
            if input.key_released(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            engine.update();
            engine.render(time, delta_time as usize);

            //triangle_scene.render(time, delta_time as usize);

            //// query keypresses this update
            //if input.key_pressed(VirtualKeyCode::A) {
            //    println!("The 'A' key was pressed on the keyboard");
            //}

            //// query the change in mouse this update
            //let mouse_diff = input.mouse_diff();
            //if mouse_diff != (0.0, 0.0) {
            //    println!("The mouse diff is: {:?}", mouse_diff);
            //    println!("The mouse position is: {:?}", input.mouse());
            //}
        }

        last_time = now;
    });
}
