use glam::{Vec2, Vec3};
use windows::Win32::{
    Foundation::*,
    Graphics::{Direct3D::*, Direct3D11::*, Dxgi::Common::*, Dxgi::*},
};

use crate::render_backend::{
    backend::Backend,
    gpu_buffer::GPUBuffer,
    mesh::*,
    render_pass::RenderPass,
    shader::Shader,
    texture::{Texture2D, TextureDescBuilder},
};

#[repr(C)]
struct QuadVertex {
    position: [f32; 3],
    uv: [f32; 2],
}

#[derive(Default)]
pub struct SimpleTriangleScene {
    pub render_passes: Vec<RenderPass>,
    pub backend: Option<Backend>,
    pub meshes: Vec<GpuMesh>,
    pub vertex_buffer: Option<GPUBuffer>,
    pub index_buffer: Option<GPUBuffer>,
    pub quad_vertex_buffer: Option<GPUBuffer>,
    pub quad_index_buffer: Option<GPUBuffer>,
    pub quad_mesh: Option<GpuMesh>,
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
                position: Vec3::from([0.0, 0.5, 0.0]),
                normal: Vec3::new(0.0, 0.0, 1.0),
                uv: Vec2::new(0.5, 1.0),
            },
            Vertex {
                position: Vec3::new(0.45, -0.5, 0.0),
                normal: Vec3::new(0.0, 0.0, 1.0),
                uv: Vec2::new(1.0, 0.0),
            },
            Vertex {
                position: Vec3::new(-0.45, -0.5, 0.0),
                normal: Vec3::new(0.0, 0.0, 1.0),
                uv: Vec2::new(0.0, 0.0),
            },
        ];

        let cpu_mesh = CpuMesh {
            vertices,
            indices: vec![0, 1, 2],
        };

        let (gpu_meshes, vertex_buffer, index_buffer) =
            upload_meshes(&backend, &[cpu_mesh]).expect("Uploading triangle mesh");

        let quad_vertices = vec![
            Vertex {
                position: Vec3::new(-1.0, -1.0, 0.0),
                normal: Vec3::new(0.0, 0.0, 1.0),
                uv: Vec2::new(0.0, 1.0),
            },
            Vertex {
                position: Vec3::new(-1.0, 1.0, 0.0),
                normal: Vec3::new(0.0, 0.0, 1.0),
                uv: Vec2::new(0.0, 0.0),
            },
            Vertex {
                position: Vec3::new(1.0, -1.0, 0.0),
                normal: Vec3::new(0.0, 0.0, 1.0),
                uv: Vec2::new(1.0, 1.0),
            },
            Vertex {
                position: Vec3::new(1.0, 1.0, 0.0),
                normal: Vec3::new(0.0, 0.0, 1.0),
                uv: Vec2::new(1.0, 0.0),
            },
        ];

        let quad_indices = vec![0, 1, 2, 2, 1, 3];

        let quad_cpu_mesh = CpuMesh {
            vertices: quad_vertices,
            indices: quad_indices,
        };

        let (gpu_quad_mesh, quad_vertex_buffer, quad_index_buffer) =
            upload_meshes(&backend, &[quad_cpu_mesh]).expect("Creating quad mesh");

        unsafe {
            backend
                .device_context
                .IASetPrimitiveTopology(D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST)
        }

        let mesh_input_desc = [
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
                SemanticName: PSTR(b"NORMAL\0".as_ptr() as _),
                SemanticIndex: 0,
                Format: DXGI_FORMAT_R32G32B32_FLOAT,
                InputSlot: 0,
                AlignedByteOffset: 12,
                InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
                InstanceDataStepRate: 0,
            },
            D3D11_INPUT_ELEMENT_DESC {
                SemanticName: PSTR(b"TEXCOORD\0".as_ptr() as _),
                SemanticIndex: 0,
                Format: DXGI_FORMAT_R32G32_FLOAT,
                InputSlot: 0,
                AlignedByteOffset: 24,
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

        let sampler_state_desc = D3D11_SAMPLER_DESC {
            Filter: D3D11_FILTER_MIN_MAG_MIP_LINEAR,
            AddressU: D3D11_TEXTURE_ADDRESS_WRAP,
            AddressV: D3D11_TEXTURE_ADDRESS_WRAP,
            AddressW: D3D11_TEXTURE_ADDRESS_WRAP,
            MipLODBias: 0.0,
            MaxAnisotropy: 1,
            ComparisonFunc: D3D11_COMPARISON_ALWAYS,
            BorderColor: [0.0, 0.0, 0.0, 0.0],
            MinLOD: 0.0,
            MaxLOD: D3D11_FLOAT32_MAX,
        };

        let sampler_state = unsafe {
            backend
                .device
                .CreateSamplerState(&sampler_state_desc)
                .expect("Creating sampler")
        };

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
                &mesh_input_desc,
                std::mem::size_of::<Vertex>() as u32,
            )
            .pixel_shader(
                Shader::pixel_shader(&backend, "gbuffer.hlsl", "pixel")
                    .expect("Create pixel shader"),
            )
            .execution(Box::new(move |pass, backend, mesh: &GpuMesh| {
                pass.clear(backend).expect("Clearing rtv");

                unsafe {
                    backend
                        .device_context
                        .DrawIndexed(mesh.num_indices, mesh.index_offset, 0);
                }

                Ok(())
            }));

        let gbuffer_combination_pass = RenderPass::new()
            .enable_depth(true)
            .depth_state(depth_stencil_state)
            .shader_resource(position_srv)
            .shader_resource(albedo_srv)
            .shader_resource(normal_srv)
            .sampler_state(sampler_state)
            .render_target(backbuffer_rtv)
            .vertex_shader(
                &backend,
                Shader::vertex_shader(&backend, "vertex_shader.hlsl", "main")
                    .expect("Create vertex shader"),
                &mesh_input_desc,
                std::mem::size_of::<Vertex>() as u32,
            )
            .pixel_shader(
                Shader::pixel_shader(&backend, "fragment_shader.hlsl", "main")
                    .expect("Creating pixel shader"),
            )
            .execution(Box::new(move |pass, backend, mesh| {
                pass.clear(backend).expect("Clearing rtv");

                unsafe {
                    backend
                        .device_context
                        .DrawIndexed(mesh.num_indices, mesh.index_offset, 0);
                }

                Ok(())
            }));

        SimpleTriangleScene {
            render_passes: vec![gbuffer_pass, gbuffer_combination_pass],
            backend: Some(backend),
            meshes: gpu_meshes,
            vertex_buffer: Some(vertex_buffer),
            index_buffer: Some(index_buffer),
            quad_mesh: Some(gpu_quad_mesh[0]),
            quad_vertex_buffer: Some(quad_vertex_buffer),
            quad_index_buffer: Some(quad_index_buffer),
        }
    }

    pub fn render(&self) {
        if self.render_passes.is_empty() || self.backend.is_none() {
            return;
        }
        let backend = self.backend.as_ref().unwrap();

        unsafe {
            backend.device_context.IASetIndexBuffer(
                self.index_buffer.as_ref().unwrap().buffer.clone(),
                DXGI_FORMAT_R32_UINT,
                0,
            );
        }

        unsafe {
            backend.device_context.IASetVertexBuffers(
                0,
                1,
                &Some(self.vertex_buffer.as_ref().unwrap().buffer.clone()),
                [std::mem::size_of::<Vertex>() as u32].as_ptr(),
                [0].as_ptr(),
            )
        }

        for mesh in &self.meshes {
            self.render_passes[0]
                .execute(backend, mesh)
                .expect("Execute gbuffer pass");
        }

        unsafe {
            backend.device_context.IASetIndexBuffer(
                self.quad_index_buffer.as_ref().unwrap().buffer.clone(),
                DXGI_FORMAT_R32_UINT,
                0,
            );
        }

        unsafe {
            backend.device_context.IASetVertexBuffers(
                0,
                1,
                &Some(self.quad_vertex_buffer.as_ref().unwrap().buffer.clone()),
                [std::mem::size_of::<Vertex>() as u32].as_ptr(),
                [0].as_ptr(),
            )
        }

        self.render_passes[1]
            .execute(backend, &self.quad_mesh.as_ref().unwrap())
            .expect("Execute gbuffer combine pass");

        unsafe {
            backend
                .swap_chain
                .Present(0, 0)
                .expect("Presenting swapchain");
        }
    }
}
