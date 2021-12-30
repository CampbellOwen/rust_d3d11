use windows::Win32::{
    Foundation::*,
    Graphics::{Direct3D::*, Direct3D11::*, Dxgi::Common::*, Dxgi::*},
};

use crate::{
    render_backend::{
        backend::{Backend, ResourceView},
        render_stage::RenderStage,
        texture::Texture,
    },
    vertex_colour_stage::create_vertex_colour_stage,
};

#[derive(Default, Debug, Clone, Copy)]
#[repr(C)]
pub struct Vertex {
    x: f32,
    y: f32,
    z: f32,
    colour: [f32; 4],
}
#[derive(Default)]
pub struct SimpleTriangleScene<'a> {
    pub buffers: Vec<ResourceView>,
    pub render_stage: Option<RenderStage<'a>>,
    pub backend: Option<Backend>,
    pub vertices: Vec<Vertex>,
    pub vertex_buffer: Option<ID3D11Buffer>,
}

impl<'a> SimpleTriangleScene<'a> {
    pub fn new(hwnd: HWND) -> Self {
        let swap_chain_desc = DXGI_SWAP_CHAIN_DESC {
            BufferDesc: DXGI_MODE_DESC {
                Format: DXGI_FORMAT_R8G8B8A8_UNORM,
                ..Default::default()
            },
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                ..Default::default()
            },
            BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
            BufferCount: 2,
            OutputWindow: hwnd as isize,
            Windowed: true.into(),
            SwapEffect: DXGI_SWAP_EFFECT_FLIP_DISCARD,
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

        let mut backbuffer_desc = Default::default();
        unsafe { backbuffer.GetDesc(&mut backbuffer_desc) }

        let backend = Backend::new(device, context, swapchain);

        let bb_tex = Texture::from_swapchain(backbuffer, backbuffer_desc);

        let backbuffer_rtv = backend
            .render_target_view(&bb_tex, None)
            .expect("Create backbuffer rtv");

        let width = 800.0;
        let height = 600.0;

        let mut viewport = D3D11_VIEWPORT {
            TopLeftX: 0.0,
            TopLeftY: 0.0,
            Width: width,
            Height: height,
            ..Default::default()
        };

        unsafe { backend.device_context.RSSetViewports(1, &mut viewport) }

        let vertices = vec![
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
            backend
                .device
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
            backend
                .device_context
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
            backend.device_context.Unmap(&vertex_buffer, 0);
        }

        unsafe {
            backend
                .device_context
                .IASetPrimitiveTopology(D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST)
        }

        let buffers = vec![backbuffer_rtv];

        SimpleTriangleScene {
            buffers,
            render_stage: None,
            backend: Some(backend),
            vertices,
            vertex_buffer: Some(vertex_buffer),
        }
    }

    pub fn setup(&'a mut self) {
        if let Some(backend) = &self.backend {
            self.render_stage = Some(create_vertex_colour_stage(&backend, &self.buffers[0]));
        }
    }

    pub fn render(&self) {
        if self.render_stage.is_none() || self.backend.is_none() {
            return;
        }
        let backend = self.backend.as_ref().unwrap();

        let strides = [std::mem::size_of::<Vertex>() as u32];
        let offsets = [0];
        unsafe {
            backend.device_context.IASetVertexBuffers(
                0,
                1,
                &self.vertex_buffer,
                strides.as_ptr(),
                offsets.as_ptr(),
            )
        }

        if let Some(render_stage) = &self.render_stage {
            render_stage.bind(backend).expect("Binding shader stage");

            unsafe {
                backend.device_context.Draw(self.vertices.len() as u32, 0);
            }

            unsafe {
                backend
                    .swap_chain
                    .Present(0, 0)
                    .expect("Presenting swapchain");
            }
        }
    }
}
