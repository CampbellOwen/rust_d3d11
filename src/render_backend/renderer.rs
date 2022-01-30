use std::collections::HashMap;

use windows::Win32::Foundation::*;
use windows::Win32::Graphics::{Direct3D::*, Direct3D11::*, Dxgi::Common::*};

use crate::{
    object::{Flag, GameObject},
    scene::Scene,
};

use super::mesh::{GpuMesh, Vertex};
use super::shader::Shader;
use super::{
    backend::Backend,
    render_pass::RenderPass,
    texture::{Tex, Tex2D, TextureDescBuilder},
};
use crate::atmosphere::precompute::{precompute_textures, AtmosphericConstants};

pub trait Renderer {
    fn render(&self, backend: &Backend, scene: &mut Scene, time: usize, delta_time: usize);
}

pub struct GBufferRTV {
    pub albedo: ID3D11RenderTargetView,
    pub position: ID3D11RenderTargetView,
    pub normal: ID3D11RenderTargetView,
}

pub struct BasicRenderer {
    pub depth_stencil_view: ID3D11DepthStencilView,
    pub depth_state: ID3D11DepthStencilState,
    pub gbuffer_write_pass: RenderPass,
    pub combination_pass: RenderPass,
    pub backbuffer_rtv: ID3D11RenderTargetView,
    pub gbuffer: GBufferRTV,
}

impl BasicRenderer {
    pub fn new(backend: &Backend, width: u32, height: u32) -> BasicRenderer {
        let viewport = D3D11_VIEWPORT {
            TopLeftX: 0.0,
            TopLeftY: 0.0,
            Width: width as f32,
            Height: height as f32,
            MaxDepth: 1.0,
            MinDepth: 0.0,
            ..Default::default()
        };

        unsafe { backend.device_context.RSSetViewports(1, &viewport) }
        let backbuffer = backend.backbuffer(0).expect("Get backbuffer texture");

        let backbuffer_rtv = backend
            .render_target_view(&backbuffer, None)
            .expect("Create backbuffer rtv");

        unsafe {
            backend
                .device_context
                .IASetPrimitiveTopology(D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST)
        }

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

        let depth_texture = Tex2D::new(
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

        let position_texture = Tex2D::new(
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

        let albedo_texture = Tex2D::new(
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

        let normal_texture = Tex2D::new(
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

        let gbuffer_write_pass = RenderPass::new()
            .enable_depth(true)
            .depth_state(depth_stencil_state.clone())
            .depth_stencil_view(depth_stencil_view.clone())
            .render_target(position_rtv.clone())
            .render_target(albedo_rtv.clone())
            .render_target(normal_rtv.clone())
            .sampler_state(sampler_state.clone())
            .clear_rtv(true)
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
            .execution(Box::new(move |_, backend, num_vertices: u32| {
                unsafe {
                    backend.device_context.DrawIndexed(num_vertices, 0, 0);
                }

                Ok(())
            }));

        let consts = AtmosphericConstants::default();
        let (transmittance, irradiance) = precompute_textures(backend, consts);

        let transmittance_srv = backend
            .shader_resource_view(&transmittance, None)
            .expect("Create transmittance srv");

        let irradiance_srv = backend
            .shader_resource_view(&irradiance, None)
            .expect("Create irradiance srv");

        let gbuffer_combination_pass = RenderPass::new()
            .enable_depth(true)
            .depth_state(depth_stencil_state.clone())
            .shader_resource(position_srv)
            .shader_resource(albedo_srv)
            .shader_resource(normal_srv)
            .shader_resource(transmittance_srv)
            .shader_resource(irradiance_srv)
            .sampler_state(sampler_state)
            .render_target(backbuffer_rtv.clone())
            .clear_rtv(true)
            .vertex_shader(
                &backend,
                Shader::vertex_shader(&backend, "vertex_shader.hlsl", "main")
                    .expect("Create vertex shader"),
                &[],
                0,
            )
            .pixel_shader(
                Shader::pixel_shader(&backend, "fragment_shader.hlsl", "main")
                    .expect("Creating pixel shader"),
            )
            .execution(Box::new(move |_, backend, _| {
                unsafe {
                    backend.device_context.Draw(6, 0);
                }

                Ok(())
            }));

        BasicRenderer {
            depth_stencil_view,
            backbuffer_rtv,
            gbuffer_write_pass,
            combination_pass: gbuffer_combination_pass,
            gbuffer: GBufferRTV {
                albedo: albedo_rtv,
                position: position_rtv,
                normal: normal_rtv,
            },
            depth_state: depth_stencil_state,
        }
    }
}

impl Renderer for BasicRenderer {
    fn render(&self, backend: &Backend, scene: &mut Scene, time: usize, delta_time: usize) {
        scene.camera.bind(backend);

        let mut opaque_objects: Vec<_> = scene
            .objects
            .iter_mut()
            .filter(|obj| obj.flags().contains(&Flag::Opaque))
            .collect();

        opaque_objects.iter_mut().for_each(|object| {
            if let Some(mesh) = object.mesh() {
                backend.unbind_shader_resources();
                object.bind(backend);

                self.gbuffer_write_pass
                    .execute(backend, mesh.num_indices)
                    .expect("Execute gbuffer pass")
            }
        });

        self.combination_pass
            .execute(backend, 6)
            .expect("Combine gbuffer pass");

        unsafe {
            backend
                .swap_chain
                .Present(1, 0)
                .expect("Presenting swapchain");
        }
    }
}
