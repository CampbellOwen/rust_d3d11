use std::borrow::Borrow;

use glam::Vec3;
use windows::Win32::Graphics::{
    Direct3D::D3D11_SRV_DIMENSION_BUFFEREX, Direct3D11::*, Dxgi::Common::*,
};

use crate::render_backend::{backend::Backend, gpu_buffer::GPUBuffer, shader::*, texture::*};

use std::ffi::{CStr, CString};

#[repr(C)]
pub struct AtmosphericConstants {
    beta_rayleigh: Vec3,
    num_scattering: u32,
    wave_lengths: Vec3,
    mu_s_min: f32,
    solar_irradiance: Vec3,
    atmos_bottom: f32,
    atmos_top: f32,
    h_m: f32,
    h_r: f32,
}

impl Default for AtmosphericConstants {
    fn default() -> Self {
        Self {
            beta_rayleigh: Vec3::new(5.8e-3, 1.35e-2, 3.31e-2),
            wave_lengths: Vec3::new(680., 550., 440.),
            solar_irradiance: Vec3::new(1.0, 1.0, 1.0),
            atmos_bottom: 6360.0,
            atmos_top: 6420.0,
            h_m: 1.2,
            h_r: 8.0,
            num_scattering: Default::default(),
            mu_s_min: -0.2,
        }
    }
}

pub fn precompute_textures(
    backend: &Backend,
    constants: AtmosphericConstants,
) -> (Tex2D, Tex2D, Tex3D) {
    let cbuffer =
        GPUBuffer::constant_buffer(backend, std::mem::size_of::<AtmosphericConstants>() as u32)
            .expect("Create cbuffer");

    //let buffer = GPUBuffer::structured_buffer::<AtmosphericConstants>(&backend, 1, false)
    //    .expect("Create constants buffer");

    {
        let mapped = cbuffer.map(backend).expect("Mapping buffer");
        mapped.copy_from(&[constants]);
    }

    let transmittance_buffer = GPUBuffer::structured_buffer::<Vec3>(backend, 256 * 64, true)
        .expect("Create transmittance buffer");

    let transmittance_uav = backend
        .unordered_access_view_buffer(&transmittance_buffer, None)
        .expect("Create UAV");

    let transmittance_srv = backend
        .shader_resource_view_buffer(&transmittance_buffer, None)
        .expect("Create srv");

    let irradiance_buffer = GPUBuffer::structured_buffer::<Vec3>(backend, 64 * 16, true)
        .expect("Create irradiance buffer");

    let irradiance_uav = backend
        .unordered_access_view_buffer(&irradiance_buffer, None)
        .expect("irradiance uav");

    let irradiance_srv = backend
        .shader_resource_view_buffer(&irradiance_buffer, None)
        .expect("irradiance srv");

    let delta_irradiance_buffer = GPUBuffer::structured_buffer::<Vec3>(backend, 64 * 16, true)
        .expect("Create delta_irradiance buffer");

    let delta_irradiance_uav = backend
        .unordered_access_view_buffer(&delta_irradiance_buffer, None)
        .expect("delta_irradiance uav");

    let delta_irradiance_srv = backend
        .shader_resource_view_buffer(&delta_irradiance_buffer, None)
        .expect("dIr srv");

    let delta_inscatter_rayleigh_buffer =
        GPUBuffer::structured_buffer::<Vec3>(backend, 32 * 128 * 32 * 8, true)
            .expect("Create delta in scatter buffer");

    let delta_inscatter_rayleigh_uav = backend
        .unordered_access_view_buffer(&delta_inscatter_rayleigh_buffer, None)
        .expect("uav");
    let delta_inscatter_rayleigh_srv = backend
        .shader_resource_view_buffer(&delta_inscatter_rayleigh_buffer, None)
        .expect("srv");

    let delta_inscatter_mie_buffer =
        GPUBuffer::structured_buffer::<Vec3>(backend, 32 * 128 * 32 * 8, true)
            .expect("Create delta in scatter buffer");

    let delta_inscatter_mie_uav = backend
        .unordered_access_view_buffer(&delta_inscatter_mie_buffer, None)
        .expect("uav");

    let delta_inscatter_mie_srv = backend
        .shader_resource_view_buffer(&delta_inscatter_mie_buffer, None)
        .expect("srv");

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

            backend
                .device_context
                .CSSetConstantBuffers(0, 1, &Some(cbuffer.buffer.clone()));

            backend.device_context.CSSetShaderResources(
                0,
                1,
                [Some(transmittance_srv.clone())].as_ptr(),
            );

            backend.device_context.CSSetUnorderedAccessViews(
                0,
                1,
                [Some(delta_irradiance_uav.clone())].as_ptr(),
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

    let single_inscatter_shader = Shader::compute_shader(
        backend,
        "atmospheric_precompute_single_inscatter.hlsl",
        "main",
    )
    .expect("Create shader");

    // Compute Single Inscatter
    if let Shader::Compute(shader, _) = single_inscatter_shader {
        unsafe {
            backend
                .device_context
                .CSSetShader(shader.clone(), std::ptr::null(), 0);

            backend
                .device_context
                .CSSetConstantBuffers(0, 1, &Some(cbuffer.buffer.clone()));

            backend.device_context.CSSetShaderResources(
                0,
                1,
                [Some(transmittance_srv.clone())].as_ptr(),
            );

            backend.device_context.CSSetUnorderedAccessViews(
                0,
                2,
                [
                    Some(delta_inscatter_rayleigh_uav.clone()),
                    Some(delta_inscatter_mie_uav.clone()),
                ]
                .as_ptr(),
                std::ptr::null(),
            );

            backend.device_context.Dispatch(8, 128, 32);

            backend
                .device_context
                .CSSetShader(None, std::ptr::null(), 0);

            backend.unbind_shader_resources();

            backend
                .device_context
                .CSSetUnorderedAccessViews(0, 1, &None, std::ptr::null());
        }
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
    .expect("Create irradiance texture");

    let transmittance_texture_uav = backend
        .unordered_access_view(&transmittance_texture, None)
        .expect("Create texture uav");

    let copy_transmittance_shader = Shader::compute_shader(
        backend,
        "atmospheric_precompute_copy_transmittance.hlsl",
        "main",
    )
    .expect("Create shader");
    // Copy transmittance buffer to texture
    if let Shader::Compute(shader, _) = copy_transmittance_shader {
        unsafe {
            backend
                .device_context
                .CSSetShader(shader.clone(), std::ptr::null(), 0);

            backend.device_context.CSSetShaderResources(
                0,
                1,
                [Some(transmittance_srv.clone())].as_ptr(),
            );

            backend.device_context.CSSetUnorderedAccessViews(
                0,
                1,
                [Some(transmittance_texture_uav.clone())].as_ptr(),
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

    let irradiance_texture_uav = backend
        .unordered_access_view(&irradiance_texture, None)
        .expect("Create texture uav");

    let copy_irradiance_shader = Shader::compute_shader(
        backend,
        "atmospheric_precompute_copy_irradiance.hlsl",
        "main",
    )
    .expect("Create shader");
    // Copy irradiance buffer to texture
    if let Shader::Compute(shader, _) = copy_irradiance_shader {
        unsafe {
            backend
                .device_context
                .CSSetShader(shader.clone(), std::ptr::null(), 0);

            backend.device_context.CSSetShaderResources(
                0,
                1,
                [Some(delta_irradiance_srv.clone())].as_ptr(),
            );

            backend.device_context.CSSetUnorderedAccessViews(
                0,
                1,
                [Some(irradiance_texture_uav.clone())].as_ptr(),
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

    let inscatter_texture = Tex3D::new(
        backend,
        TextureDescBuilder::new()
            .bind_flags(D3D11_BIND_SHADER_RESOURCE | D3D11_BIND_UNORDERED_ACCESS)
            .format(DXGI_FORMAT_R32G32B32A32_FLOAT)
            .mip_levels(1)
            .size([256, 128, 32])
            .build_texture3d(),
    )
    .expect("Create inscatter tex");

    let inscatter_texture_uav = backend
        .unordered_access_view(&inscatter_texture, None)
        .expect("uav");
    let copy_inscatter_single_shader = Shader::compute_shader(
        backend,
        "atmospheric_precompute_copy_single_inscatter.hlsl",
        "main",
    )
    .expect("Create shader");

    // Copy inscatter buffer to texture
    if let Shader::Compute(shader, _) = copy_inscatter_single_shader {
        unsafe {
            backend
                .device_context
                .CSSetShader(shader.clone(), std::ptr::null(), 0);

            backend.device_context.CSSetShaderResources(
                0,
                2,
                [
                    Some(delta_inscatter_rayleigh_srv.clone()),
                    Some(delta_inscatter_mie_srv.clone()),
                ]
                .as_ptr(),
            );

            backend.device_context.CSSetUnorderedAccessViews(
                0,
                1,
                [Some(inscatter_texture_uav.clone())].as_ptr(),
                std::ptr::null(),
            );

            backend.device_context.Dispatch(8, 128, 32);

            backend
                .device_context
                .CSSetShader(None, std::ptr::null(), 0);

            backend.unbind_shader_resources();

            backend
                .device_context
                .CSSetUnorderedAccessViews(0, 1, &None, std::ptr::null());
        }
    }

    (transmittance_texture, irradiance_texture, inscatter_texture)
}
