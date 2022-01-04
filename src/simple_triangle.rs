use glam::{Mat4, Quat, Vec2, Vec3};
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
struct FrameConstants {
    world_view: Mat4,
}

#[repr(C)]
struct ModelConstants {
    model_world: Mat4,
}

#[derive(Default)]
pub struct SimpleTriangleScene {
    pub render_passes: Vec<RenderPass>,
    pub backend: Option<Backend>,
    pub meshes: Vec<GpuMesh>,
    pub depth_stencil_view: Option<ID3D11DepthStencilView>,
    pub vertex_buffer: Option<GPUBuffer>,
    pub index_buffer: Option<GPUBuffer>,
    pub quad_vertex_buffer: Option<GPUBuffer>,
    pub quad_index_buffer: Option<GPUBuffer>,
    pub quad_mesh: Option<GpuMesh>,
    pub frame_constants: Option<GPUBuffer>,
    pub model_constants: Option<GPUBuffer>,
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
            MaxDepth: 1.0,
            MinDepth: 0.0,
            ..Default::default()
        };

        unsafe { backend.device_context.RSSetViewports(1, &viewport) }

        //let cube = CpuMesh::from_obj("F:\\Models\\cube.obj").expect("Load obj");
        //let cube = CpuMesh::from_obj("F:\\Models\\JapaneseTemple\\model_triangulated.obj")
        let cube = CpuMesh::from_obj("F:\\Models\\lost-empire\\lost_empire_triangulated.obj")
            .expect("Load obj");

        let (gpu_meshes, vertex_buffer, index_buffer) =
            GpuMesh::from_meshes(&backend, cube.as_slice()).expect("Uploading triangle mesh");

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
            GpuMesh::from_meshes(&backend, &[quad_cpu_mesh]).expect("Creating quad mesh");

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

        let depth_texture = Texture2D::new(
            &backend,
            TextureDescBuilder::new()
                .size([width as u32, height as u32, 0])
                .mip_levels(1)
                .format(DXGI_FORMAT_D24_UNORM_S8_UINT)
                .bind_flags(D3D11_BIND_DEPTH_STENCIL)
                .build_texture2d(),
        )
        .expect("Create depth texture");

        let depth_stencil_view = backend
            .depth_stencil_view(&depth_texture, None)
            .expect("Create depth stencil view");

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
            .depth_stencil_view(depth_stencil_view.clone())
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
            .execution(Box::new(move |_, backend, mesh: &GpuMesh| {
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
            .execution(Box::new(move |_, backend, mesh| {
                unsafe {
                    backend
                        .device_context
                        .DrawIndexed(mesh.num_indices, mesh.index_offset, 0);
                }

                Ok(())
            }));

        let frame_constants =
            GPUBuffer::constant_buffer(&backend, std::mem::size_of::<FrameConstants>() as u32)
                .expect("Create frame constant buffer");

        let model_constants =
            GPUBuffer::constant_buffer(&backend, std::mem::size_of::<ModelConstants>() as u32)
                .expect("Create model constant buffer");

        {
            let mapped_frame = frame_constants
                .map(&backend)
                .expect("Map frame constant buffer");
            let mapped_model = model_constants
                .map(&backend)
                .expect("Map model constant buffer");

            let camera_pos = Vec3::new(0.0, 80.0, -70.0);
            let focal_point = Vec3::new(0.0, 20.0, 20.0);

            let view_dir = focal_point - camera_pos;

            let up = view_dir.cross(Vec3::new(1.0, 0.0, 0.0));

            mapped_frame.copy_from(&[FrameConstants {
                world_view: Mat4::perspective_lh(
                    std::f32::consts::PI / 4.0,
                    800.0 / 600.0,
                    0.001,
                    1000.0,
                ) * Mat4::look_at_lh(camera_pos, focal_point, up),
            }]);

            mapped_model.copy_from(&[ModelConstants {
                model_world: Mat4::from_translation(Vec3::new(0.0, 0.0, 12.0)),
            }])
        }

        SimpleTriangleScene {
            render_passes: vec![gbuffer_pass, gbuffer_combination_pass],
            backend: Some(backend),
            meshes: gpu_meshes,
            vertex_buffer: Some(vertex_buffer),
            index_buffer: Some(index_buffer),
            quad_mesh: Some(gpu_quad_mesh[0]),
            quad_vertex_buffer: Some(quad_vertex_buffer),
            quad_index_buffer: Some(quad_index_buffer),
            frame_constants: Some(frame_constants),
            model_constants: Some(model_constants),
            depth_stencil_view: Some(depth_stencil_view),
        }
    }

    pub fn render(&self, time: usize, _: usize) {
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

        let model_constant = self.model_constants.as_ref().unwrap();

        {
            let mapped = model_constant
                .map(backend)
                .expect("map model constant buffer");

            let data = ModelConstants {
                model_world: Mat4::from_scale_rotation_translation(
                    Vec3::new(0.6, 0.6, 0.6),
                    Quat::from_mat4(
                        &(
                            Mat4::from_rotation_y((time as f32 + 100 as f32) / 1000.0)
                            //Mat4::from_rotation_y(std::f32::consts::PI / 2.0)
                            //     * Mat4::from_rotation_x(time as f32 / 300.0)
                            //     * Mat4::from_rotation_z((time as f32 + 200 as f32) / 300.0)
                        ),
                    ),
                    Vec3::new(0.0, -10.0, 50.0),
                ),
            };

            mapped.copy_from(&[data]);
        }

        let constant_buffers = [
            Some(self.frame_constants.as_ref().unwrap().buffer.clone()),
            Some(self.model_constants.as_ref().unwrap().buffer.clone()),
        ];

        unsafe {
            backend.device_context.PSSetConstantBuffers(
                0,
                constant_buffers.len() as u32,
                constant_buffers.as_ptr(),
            );
            backend.device_context.VSSetConstantBuffers(
                0,
                constant_buffers.len() as u32,
                constant_buffers.as_ptr(),
            );
        }

        self.meshes.iter().enumerate().for_each(|(index, mesh)| {
            self.render_passes[0]
                .execute(backend, mesh, index == 0)
                .expect("Execute gbuffer pass");
        });

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
            .execute(backend, &self.quad_mesh.as_ref().unwrap(), true)
            .expect("Execute gbuffer combine pass");

        unsafe {
            backend
                .swap_chain
                .Present(1, 0)
                .expect("Presenting swapchain");
        }
    }
}
