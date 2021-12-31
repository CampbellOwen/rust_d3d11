use std::marker::PhantomData;

use windows::Win32::{
    Foundation::*,
    Graphics::{Direct3D::*, Direct3D11::*, Dxgi::Common::*, Dxgi::*},
};

use crate::{
    render_backend::{
        backend::Backend,
        gpu_buffer::GPUBuffer,
        render_pass::RenderPass,
        shader::Shader,
        texture::{Texture2D, TextureDescBuilder},
    },
    simple_gbuffer_pass::create_gbuffer_pass,
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
pub struct SimpleTriangleScene {
    pub render_passes: Vec<RenderPass>,
    pub backend: Option<Backend>,
    pub vertices: Vec<Vertex>,
    pub vertex_buffer: Option<GPUBuffer>,
}

impl SimpleTriangleScene {
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

        let backend = Backend::new(device, context, swapchain);

        let backbuffer = backend.backbuffer(0).expect("Get backbuffer texture");

        let backbuffer_rtv = backend
            .render_target_view(&backbuffer, None)
            .expect("Create backbuffer rtv");

        let width = 800.0;
        let height = 600.0;

        let viewport = D3D11_VIEWPORT {
            TopLeftX: 0.0,
            TopLeftY: 0.0,
            Width: width,
            Height: height,
            ..Default::default()
        };

        unsafe { backend.device_context.RSSetViewports(1, &viewport) }

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

        let vertex_buffer = GPUBuffer::new(
            &backend,
            D3D11_BUFFER_DESC {
                ByteWidth: std::mem::size_of::<Vertex>() as u32 * vertices.len() as u32,
                Usage: D3D11_USAGE_DYNAMIC,
                BindFlags: D3D11_BIND_VERTEX_BUFFER,
                CPUAccessFlags: D3D11_CPU_ACCESS_WRITE,
                ..Default::default()
            },
        )
        .expect("Creating vertex buffer");

        {
            let mapped_buffer = vertex_buffer.map(&backend).expect("Mapping vertex buffer");

            mapped_buffer.copy_from(
                vertices.as_ptr(),
                std::mem::size_of::<Vertex>() * vertices.len(),
            );
        }

        unsafe {
            backend
                .device_context
                .IASetPrimitiveTopology(D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST)
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

        let depth_stencil_desc = D3D11_DEPTH_STENCIL_DESC {
            DepthEnable: true.into(),
            DepthWriteMask: D3D11_DEPTH_WRITE_MASK_ALL,
            DepthFunc: D3D11_COMPARISON_LESS,
            StencilEnable: true.into(),
            StencilReadMask: 0xFF,
            StencilWriteMask: 0xFF,
            FrontFace: D3D11_DEPTH_STENCILOP_DESC {
                StencilFailOp: D3D11_STENCIL_OP_KEEP,
                StencilDepthFailOp: D3D11_STENCIL_OP_INCR,
                StencilPassOp: D3D11_STENCIL_OP_KEEP,
                StencilFunc: D3D11_COMPARISON_ALWAYS,
            },
            BackFace: D3D11_DEPTH_STENCILOP_DESC {
                StencilFailOp: D3D11_STENCIL_OP_KEEP,
                StencilDepthFailOp: D3D11_STENCIL_OP_DECR,
                StencilPassOp: D3D11_STENCIL_OP_KEEP,
                StencilFunc: D3D11_COMPARISON_ALWAYS,
            },
        };

        let depth_stencil_state = unsafe {
            backend
                .device
                .CreateDepthStencilState(&depth_stencil_desc)
                .expect("Create depth stencil state")
        };

        let position_texture = Texture2D::new(
            &backend,
            TextureDescBuilder::new()
                .size([width as u32, height as u32, 0])
                .mip_levels(1)
                .format(DXGI_FORMAT_R32G32B32A32_FLOAT)
                .bind_flags(D3D11_BIND_SHADER_RESOURCE | D3D11_BIND_RENDER_TARGET)
                .build_texture2d(),
        )
        .expect("Creating position texture");
        let position_rtv = backend
            .render_target_view(&position_texture, None)
            .expect("Create position rtv");
        let position_srv = backend
            .shader_resource_view(&position_texture, None)
            .expect("Create position srv");

        let albedo_texture = Texture2D::new(
            &backend,
            TextureDescBuilder::new()
                .size([width as u32, height as u32, 0])
                .mip_levels(1)
                .format(DXGI_FORMAT_R32G32B32A32_FLOAT)
                .bind_flags(D3D11_BIND_SHADER_RESOURCE | D3D11_BIND_RENDER_TARGET)
                .build_texture2d(),
        )
        .expect("Creating albedo texture");
        let albedo_rtv = backend
            .render_target_view(&albedo_texture, None)
            .expect("Create albedo rtv");
        let albedo_srv = backend
            .shader_resource_view(&albedo_texture, None)
            .expect("Create albedo srv");

        let normal_texture = Texture2D::new(
            &backend,
            TextureDescBuilder::new()
                .size([width as u32, height as u32, 0])
                .mip_levels(1)
                .format(DXGI_FORMAT_R32G32B32A32_FLOAT)
                .bind_flags(D3D11_BIND_SHADER_RESOURCE | D3D11_BIND_RENDER_TARGET)
                .build_texture2d(),
        )
        .expect("Creating normal texture");
        let normal_rtv = backend
            .render_target_view(&normal_texture, None)
            .expect("Create normal rtv");

        let normal_srv = backend
            .shader_resource_view(&normal_texture, None)
            .expect("Create normal srv");

        let gbuffer_pass = RenderPass::new()
            .enable_depth(true)
            .depth_state(depth_stencil_state.clone())
            .render_target(position_rtv)
            .render_target(albedo_rtv)
            .render_target(normal_rtv)
            .vertex_shader(
                &backend,
                Shader::vertex_shader(&backend, "gbuffer.hlsl", "vertex")
                    .expect("Create vertex shader"),
                &input_desc,
            )
            .pixel_shader(
                Shader::pixel_shader(&backend, "gbuffer.hlsl", "pixel")
                    .expect("Create pixel shader"),
            );

        let vertex_colour_pass = RenderPass::new()
            .enable_depth(true)
            .depth_state(depth_stencil_state)
            .shader_resource(position_srv)
            .shader_resource(albedo_srv)
            .shader_resource(normal_srv)
            .render_target(backbuffer_rtv)
            .vertex_shader(
                &backend,
                Shader::vertex_shader(&backend, "vertex_shader.hlsl", "main")
                    .expect("Create vertex shader"),
                &input_desc,
            )
            .pixel_shader(
                Shader::pixel_shader(&backend, "fragment_shader.hlsl", "main")
                    .expect("Creating pixel shader"),
            );

        SimpleTriangleScene {
            render_passes: vec![gbuffer_pass, vertex_colour_pass],
            backend: Some(backend),
            vertices,
            vertex_buffer: Some(vertex_buffer),
        }
    }

    pub fn render(&self) {
        if self.render_passes.is_empty() || self.backend.is_none() {
            return;
        }
        let backend = self.backend.as_ref().unwrap();

        let strides = [std::mem::size_of::<Vertex>() as u32];
        let offsets = [0];
        unsafe {
            backend.device_context.IASetVertexBuffers(
                0,
                1,
                &Some(self.vertex_buffer.as_ref().unwrap().buffer.clone()),
                strides.as_ptr(),
                offsets.as_ptr(),
            )
        }

        let gbuffer_pass = &self.render_passes[0];
        gbuffer_pass.bind(backend).expect("Binding gbuffer pass");
        unsafe {
            backend.device_context.Draw(self.vertices.len() as u32, 0);
        }

        let vertex_colour_pass = &self.render_passes[1];
        vertex_colour_pass
            .bind(backend)
            .expect("Binding vertex colour pass");

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
