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

mod vertex_colour_stage;

static mut TRIANGLE_SCENE: SimpleTriangleScene = SimpleTriangleScene {
    buffers: Vec::new(),
    render_stage: None,
    backend: None,
    vertices: Vec::new(),
    vertex_buffer: None,
};

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

    unsafe {
        TRIANGLE_SCENE = SimpleTriangleScene::new(hwnd as HWND);
        TRIANGLE_SCENE.setup();
    }

    event_loop.run(move |event, _, control_flow| {
        // Pass every event to the WindowInputHelper.
        // It will return true when the last event has been processed and it is time to run your application logic.

        if input.update(&event) {
            if input.key_released(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            unsafe {
                TRIANGLE_SCENE.render();
            }

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
    });
}
