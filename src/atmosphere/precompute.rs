use std::borrow::Borrow;

use glam::Vec3;
use windows::Win32::Graphics::{
    Direct3D::D3D11_SRV_DIMENSION_BUFFEREX, Direct3D11::*, Dxgi::Common::*,
};

use crate::render_backend::{backend::Backend, gpu_buffer::GPUBuffer, shader::*, texture::*};

pub struct AtmosphericConstants {
    atmos_bottom: f32,
    atmos_top: f32,
    h_m: f32,
    h_r: f32,
    beta_rayleigh: Vec3,
    wave_lengths: Vec3,
    num_scattering: u32,
}

impl Default for AtmosphericConstants {
    fn default() -> Self {
        Self {
            atmos_bottom: 6360.0,
            atmos_top: 6420.0,
            h_m: 1.2,
            h_r: 8.0,
            beta_rayleigh: Vec3::new(5.8e-3, 1.35e-2, 3.31e-2),
            wave_lengths: Vec3::new(680., 550., 440.),
            num_scattering: Default::default(),
        }
    }
}

pub fn precompute_textures(backend: &Backend, constants: AtmosphericConstants) -> ID3D11Texture2D {
    let cbuffer =
        GPUBuffer::constant_buffer(backend, std::mem::size_of::<AtmosphericConstants>() as u32)
            .expect("Create cbuffer");

    //let buffer = GPUBuffer::structured_buffer::<AtmosphericConstants>(&backend, 1, false)
    //    .expect("Create constants buffer");

    {
        let mapped = cbuffer.map(backend).expect("Mapping buffer");
        mapped.copy_from(&[constants]);
    }

    let transmittance_texture = Tex2D::new(
        backend,
        TextureDescBuilder::new()
            .bind_flags(D3D11_BIND_SHADER_RESOURCE | D3D11_BIND_UNORDERED_ACCESS)
            .format(DXGI_FORMAT_R32G32B32A32_FLOAT)
            .mip_levels(1)
            .size([256, 64, 0])
            .build_texture2d(),
    )
    .expect("Create transmittance texture");
    let transmittance_uav = backend
        .unordered_access_view(&transmittance_texture, None)
        .expect("Create UAV");
    let transmittance_srv = backend
        .shader_resource_view(&transmittance_texture, None)
        .expect("SRV for transmittance");

    let irradiance_texture = Tex2D::new(
        backend,
        TextureDescBuilder::new()
            .bind_flags(D3D11_BIND_SHADER_RESOURCE | D3D11_BIND_UNORDERED_ACCESS)
            .format(DXGI_FORMAT_R32G32B32A32_FLOAT)
            .mip_levels(1)
            .size([64, 16, 0])
            .build_texture2d(),
    )
    .expect("Create irradiance texture");

    let irradiance_uav = backend
        .unordered_access_view(&irradiance_texture, None)
        .expect("irradiance uav");
    let irradiance_srv = backend
        .shader_resource_view(&irradiance_texture, None)
        .expect("Irradiance srv");

    let delta_irradiance_texture = Tex2D::new(
        backend,
        TextureDescBuilder::new()
            .bind_flags(D3D11_BIND_SHADER_RESOURCE | D3D11_BIND_UNORDERED_ACCESS)
            .format(DXGI_FORMAT_R32G32B32A32_FLOAT)
            .mip_levels(1)
            .size([64, 16, 0])
            .build_texture2d(),
    )
    .expect("Create delta_irradiance texture");

    let delta_irradiance_uav = backend
        .unordered_access_view(&delta_irradiance_texture, None)
        .expect("delta_irradiance uav");
    let delta_irradiance_srv = backend
        .shader_resource_view(&delta_irradiance_texture, None)
        .expect("delta_Irradiance srv");

    let inscatter_texture = Tex2D::new(
        backend,
        TextureDescBuilder::new()
            .bind_flags(D3D11_BIND_SHADER_RESOURCE | D3D11_BIND_UNORDERED_ACCESS)
            .format(DXGI_FORMAT_R32G32B32A32_FLOAT)
            .mip_levels(1)
            .size([64, 16, 0])
            .build_texture2d(),
    )
    .expect("Create inscatter texture");

    let inscatter_uav = backend
        .unordered_access_view(&inscatter_texture, None)
        .expect("inscatter uav");
    let inscatter_srv = backend
        .shader_resource_view(&inscatter_texture, None)
        .expect("inscatter srv");

    let delta_inscatter_texture = Tex2D::new(
        backend,
        TextureDescBuilder::new()
            .bind_flags(D3D11_BIND_SHADER_RESOURCE | D3D11_BIND_UNORDERED_ACCESS)
            .format(DXGI_FORMAT_R32G32B32A32_FLOAT)
            .mip_levels(1)
            .size([64, 16, 0])
            .build_texture2d(),
    )
    .expect("Create delta_inscatter texture");

    let delta_inscatter_uav = backend
        .unordered_access_view(&delta_inscatter_texture, None)
        .expect("delta_inscatter uav");
    let delta_inscatter_srv = backend
        .shader_resource_view(&delta_inscatter_texture, None)
        .expect("delta_inscatter srv");

    let transmittance_shader =
        Shader::compute_shader(backend, "atmospheric_precompute_transmittance.hlsl", "main")
            .expect("Create shader");

    // Compute Transmittance
    if let Shader::Compute(shader, _) = transmittance_shader {
        unsafe {
            backend
                .device_context
                .CSSetShader(shader.clone(), std::ptr::null(), 0);

            //backend
            //    .device_context
            //    .CSSetShaderResources(0, 1, &Some(constants_srv.clone()));
            backend
                .device_context
                .CSSetConstantBuffers(0, 1, &Some(cbuffer.buffer.clone()));

            backend.device_context.CSSetUnorderedAccessViews(
                0,
                1,
                &Some(transmittance_uav.clone()),
                std::ptr::null(),
            );

            backend.device_context.Dispatch(8, 64, 1);

            backend
                .device_context
                .CSSetShader(None, std::ptr::null(), 0);

            backend.unbind_shader_resources();

            backend
                .device_context
                .CSSetUnorderedAccessViews(0, 1, &None, std::ptr::null());
        }
    }

    let single_irradiance_shader = Shader::compute_shader(
        backend,
        "atmospheric_precompute_single_irradiance.hlsl",
        "main",
    )
    .expect("Create shader");

    // Compute Single Irradiance
    if let Shader::Compute(shader, _) = single_irradiance_shader {
        unsafe {
            backend
                .device_context
                .CSSetShader(shader.clone(), std::ptr::null(), 0);

            backend.device_context.CSSetShaderResources(
                0,
                1,
                [Some(transmittance_srv.clone())].as_ptr(),
            );

            backend
                .device_context
                .CSSetConstantBuffers(0, 1, &Some(cbuffer.buffer.clone()));

            backend.device_context.CSSetUnorderedAccessViews(
                0,
                1,
                &Some(delta_irradiance_uav),
                std::ptr::null(),
            );

            backend.device_context.Dispatch(2, 16, 1);

            backend
                .device_context
                .CSSetShader(None, std::ptr::null(), 0);

            backend.unbind_shader_resources();

            backend
                .device_context
                .CSSetUnorderedAccessViews(0, 1, &None, std::ptr::null());
        }
    }

    transmittance_texture.device_texture()
}
