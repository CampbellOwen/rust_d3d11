use windows::Win32::Foundation::HINSTANCE;
use windows::{
    Win32::Graphics::Direct3D::*, Win32::Graphics::Direct3D11::*, Win32::Graphics::Dxgi::Common::*,
    Win32::Graphics::Dxgi::*,
};
use winit::dpi::LogicalSize;
use winit::event::VirtualKeyCode;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

use winit::platform::windows::WindowExtWindows;

fn main() {
    let mut input = WinitInputHelper::new();

    let event_loop = EventLoop::new();
    let _window = WindowBuilder::new()
        .with_title("D3D11")
        .with_inner_size(LogicalSize::new(800, 600))
        .build(&event_loop)
        .unwrap();

    let hwnd = _window.hwnd();

    let swap_chain_desc = DXGI_SWAP_CHAIN_DESC {
        BufferDesc: DXGI_MODE_DESC {
            Format: DXGI_FORMAT_R8G8B8A8_UNORM,
            ..Default::default()
        },
        SampleDesc: DXGI_SAMPLE_DESC {
            Count: 4,
            ..Default::default()
        },
        BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
        BufferCount: 1,
        OutputWindow: hwnd as isize,
        Windowed: true.into(),
        ..Default::default()
    };

    let mut device = None;
    let mut swapchain = None;
    let mut context = None;

    unsafe {
        D3D11CreateDeviceAndSwapChain(
            None,
            D3D_DRIVER_TYPE_HARDWARE,
            None,
            D3D11_CREATE_DEVICE_DEBUG,
            std::ptr::null(),
            0,
            D3D11_SDK_VERSION,
            &swap_chain_desc,
            &mut swapchain,
            &mut device,
            std::ptr::null_mut(),
            &mut context,
        )
        .expect("Error creating device and swapchain");
    };

    let device = device.expect("Device should be created");
    let swapchain = swapchain.expect("Swapchain should be created");
    let context = context.expect("DeviceContext should be created");

    let backbuffer = unsafe {
        swapchain
            .GetBuffer::<ID3D11Texture2D>(0)
            .expect("Getting backbuffer should succeed")
    };

    let mut rtv = unsafe {
        device
            .CreateRenderTargetView(&backbuffer, std::ptr::null())
            .ok()
    };

    unsafe { context.OMSetRenderTargets(1, &mut rtv, None) };

    let rtv = rtv.expect("Create rtv for backbuffer");

    let mut viewport = D3D11_VIEWPORT {
        TopLeftX: 0.0,
        TopLeftY: 0.0,
        Width: 800.0,
        Height: 600.0,
        ..Default::default()
    };

    unsafe { context.RSGetViewports(&mut 1, &mut viewport) }

    event_loop.run(move |event, _, control_flow| {
        // Pass every event to the WindowInputHelper.
        // It will return true when the last event has been processed and it is time to run your application logic.
        if input.update(&event) {
            if input.key_released(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
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

            unsafe {
                context.ClearRenderTargetView(&rtv, [0.0, 0.2, 0.4, 1.0].as_ptr());
            }

            unsafe {
                swapchain.Present(0, 0).expect("Presenting swapchain");
            }
        }
    });
}
