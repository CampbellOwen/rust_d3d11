use windows::{
    Win32::Foundation::*, Win32::Graphics::Direct3D::Fxc::*, Win32::Graphics::Direct3D::*,
    Win32::Graphics::Direct3D11::*, Win32::Graphics::Dxgi::Common::*, Win32::Graphics::Dxgi::*,
};
use winit::dpi::LogicalSize;
use winit::event::VirtualKeyCode;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

use winit::platform::windows::WindowExtWindows;

#[derive(Default, Debug, Clone, Copy)]
#[repr(C)]
struct Vertex {
    x: f32,
    y: f32,
    z: f32,
    colour: [f32; 4],
}

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

    let rtv = unsafe {
        device
            .CreateRenderTargetView(&backbuffer, std::ptr::null())
            .ok()
    };

    unsafe { context.OMSetRenderTargets(1, &rtv, None) };

    let rtv = rtv.expect("Create rtv for backbuffer");

    let mut viewport = D3D11_VIEWPORT {
        TopLeftX: 0.0,
        TopLeftY: 0.0,
        Width: 800.0,
        Height: 600.0,
        ..Default::default()
    };

    unsafe { context.RSSetViewports(1, &mut viewport) }

    let mut vertex_shader = None;
    unsafe {
        D3DCompileFromFile(
            "vertex_shader.hlsl",
            std::ptr::null(),
            None,
            "main",
            "vs_5_0",
            D3DCOMPILE_DEBUG | D3DCOMPILE_SKIP_OPTIMIZATION,
            0,
            &mut vertex_shader,
            std::ptr::null_mut(),
        )
        .expect("Compile Vertex Shader");
    }

    let vs_blob = vertex_shader.expect("Vertex shader exists");

    let vertex_shader = unsafe {
        device
            .CreateVertexShader(vs_blob.GetBufferPointer(), vs_blob.GetBufferSize(), &None)
            .expect("Creating vertex shader")
    };

    let mut fragment_shader = None;
    unsafe {
        D3DCompileFromFile(
            "fragment_shader.hlsl",
            std::ptr::null(),
            None,
            "main",
            "ps_5_0",
            D3DCOMPILE_DEBUG | D3DCOMPILE_SKIP_OPTIMIZATION,
            0,
            &mut fragment_shader,
            std::ptr::null_mut(),
        )
        .expect("Compile Fragment Shader");
    }

    let fs_blob = fragment_shader.expect("Fragment shader exists");

    let fragment_shader = unsafe {
        device
            .CreatePixelShader(fs_blob.GetBufferPointer(), fs_blob.GetBufferSize(), &None)
            .expect("Creating fragment shader")
    };

    unsafe {
        context.VSSetShader(vertex_shader, std::ptr::null(), 0);
    };

    unsafe {
        context.PSSetShader(fragment_shader, std::ptr::null(), 0);
    }

    let vertices = [
        Vertex {
            x: 0.0,
            y: 0.5,
            z: 0.0,
            colour: [1.0, 0.0, 0.0, 1.0],
        },
        Vertex {
            x: 0.45,
            y: -0.5,
            z: 0.0,
            colour: [0.0, 1.0, 0.0, 1.0],
        },
        Vertex {
            x: -0.45,
            y: -0.5,
            z: 0.0,
            colour: [0.0, 0.0, 1.0, 1.0],
        },
    ];

    let vertex_buffer = unsafe {
        device
            .CreateBuffer(
                &D3D11_BUFFER_DESC {
                    ByteWidth: std::mem::size_of::<Vertex>() as u32 * vertices.len() as u32,
                    Usage: D3D11_USAGE_DYNAMIC,
                    BindFlags: D3D11_BIND_VERTEX_BUFFER,
                    CPUAccessFlags: D3D11_CPU_ACCESS_WRITE,
                    ..Default::default()
                },
                std::ptr::null(),
            )
            .expect("Create vertex buffer")
    };

    let mapped_buffer = unsafe {
        context
            .Map(&vertex_buffer, 0, D3D11_MAP_WRITE_DISCARD, 0)
            .expect("Map vertex buffer")
    };

    unsafe {
        mapped_buffer.pData.copy_from(
            vertices.as_ptr() as _,
            std::mem::size_of::<Vertex>() * vertices.len(),
        );
    }

    unsafe {
        context.Unmap(&vertex_buffer, 0);
    }

    let input_desc = [
        D3D11_INPUT_ELEMENT_DESC {
            SemanticName: PSTR(b"POSITION\0".as_ptr() as _),
            SemanticIndex: 0,
            Format: DXGI_FORMAT_R32G32B32_FLOAT,
            InputSlot: 0,
            AlignedByteOffset: 0,
            InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
            InstanceDataStepRate: 0,
        },
        D3D11_INPUT_ELEMENT_DESC {
            SemanticName: PSTR(b"COLOR\0".as_ptr() as _),
            SemanticIndex: 0,
            Format: DXGI_FORMAT_R32G32B32A32_FLOAT,
            InputSlot: 0,
            AlignedByteOffset: 12,
            InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
            InstanceDataStepRate: 0,
        },
    ];

    let input_layout = unsafe {
        device
            .CreateInputLayout(
                input_desc.as_ptr(),
                2,
                vs_blob.GetBufferPointer(),
                vs_blob.GetBufferSize(),
            )
            .expect("Create input layout")
    };

    unsafe {
        context.IASetInputLayout(&input_layout);
    }

    let strides = [std::mem::size_of::<Vertex>() as u32];
    let offsets = [0];
    unsafe {
        context.IASetVertexBuffers(
            0,
            1,
            &Some(vertex_buffer),
            strides.as_ptr(),
            offsets.as_ptr(),
        )
    }

    unsafe { context.IASetPrimitiveTopology(D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST) }

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

                context.Draw(vertices.len() as u32, 0);
            }

            unsafe {
                swapchain.Present(0, 0).expect("Presenting swapchain");
            }
        }
    });
}
