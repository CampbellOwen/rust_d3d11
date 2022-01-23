use std::borrow::Borrow;

use glam::Vec3;
use windows::Win32::Graphics::{
    Direct3D::D3D11_SRV_DIMENSION_BUFFEREX, Direct3D11::*, Dxgi::Common::*,
};

use crate::render_backend::{backend::Backend, gpu_buffer::GPUBuffer, shader::*, texture::*};

pub struct AtmosphericConstants {
    atmos_bottom: f32,
    atmos_top: f32,
    beta_rayleigh: Vec3,
    wave_lengths: Vec3,
    num_scattering: u32,
}

impl Default for AtmosphericConstants {
    fn default() -> Self {
        Self {
            atmos_bottom: 6360.0,
            atmos_top: 6420.0,
            beta_rayleigh: Vec3::new(5.8e-6, 13.5e-6, 33.1e-6),
            wave_lengths: Vec3::new(680., 550., 440.),
            num_scattering: Default::default(),
        }
    }
}

pub fn precompute_textures(backend: &Backend, constants: AtmosphericConstants) -> ID3D11Texture2D {
    let buffer = GPUBuffer::structured_buffer::<AtmosphericConstants>(&backend, 1, false)
        .expect("Create constants buffer");

    {
        let mapped = buffer.map(backend).expect("Mapping buffer");
        mapped.copy_from(&[constants]);
    }

    let constants_srv_desc = D3D11_SHADER_RESOURCE_VIEW_DESC {
        Format: DXGI_FORMAT_UNKNOWN,
        ViewDimension: D3D11_SRV_DIMENSION_BUFFEREX,
        Anonymous: D3D11_SHADER_RESOURCE_VIEW_DESC_0 {
            BufferEx: D3D11_BUFFEREX_SRV {
                FirstElement: 0,
                NumElements: 1,
                Flags: Default::default(),
            },
        },
    };

    let constants_srv = unsafe {
        backend
            .device
            .CreateShaderResourceView(buffer.buffer.clone(), &constants_srv_desc)
            .expect("Create srv")
    };

    let transmittance_texture = Tex2D::new(
        backend,
        TextureDescBuilder::new()
            .bind_flags(D3D11_BIND_SHADER_RESOURCE | D3D11_BIND_UNORDERED_ACCESS)
            .format(DXGI_FORMAT_R32G32B32A32_FLOAT)
            .mip_levels(1)
            .size([64, 256, 0])
            .build_texture2d(),
    )
    .expect("Create transmittance texture");

    let transmittance_uav = backend
        .unordered_access_view(&transmittance_texture, None)
        .expect("Create UAV");

    let compute_shader = Shader::compute_shader(backend, "atmospheric_precompute.hlsl", "main")
        .expect("Create shader");

    if let Shader::Compute(shader, _) = compute_shader {
        unsafe {
            backend
                .device_context
                .CSSetShader(shader.clone(), std::ptr::null(), 0);

            backend
                .device_context
                .CSSetShaderResources(0, 1, &Some(constants_srv));

            backend.device_context.CSSetUnorderedAccessViews(
                0,
                1,
                &Some(transmittance_uav),
                std::ptr::null(),
            );

            backend.device_context.Dispatch(2, 256, 1);

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
