use windows::core::*;
use windows::Win32::Graphics::Direct3D::{Fxc::*, ID3DBlob};
use windows::Win32::Graphics::Direct3D11::*;

use super::backend::Backend;

fn compile_shader(path: &str, entry_point: &str, target: &str) -> Result<ID3DBlob> {
    let flags = if cfg!(debug_assertions) {
        D3DCOMPILE_DEBUG | D3DCOMPILE_SKIP_OPTIMIZATION
    } else {
        0
    };

    let mut shader_blob = None;

    unsafe {
        D3DCompileFromFile(
            path,
            std::ptr::null(),
            None,
            entry_point,
            target,
            flags,
            0,
            &mut shader_blob,
            std::ptr::null_mut(),
        )?
    }

    shader_blob.ok_or_else(|| Error::fast_error(HRESULT::from_win32(0x80070057)))
}

pub enum Shader {
    Vertex(ID3D11VertexShader, ID3DBlob),
    Pixel(ID3D11PixelShader, ID3DBlob),
}

impl Shader {
    pub fn pixel_shader(backend: &Backend, path: &str, entry_point: &str) -> Result<Shader> {
        let shader_blob = compile_shader(path, entry_point, "ps_5_0")?;

        let shader = unsafe {
            backend.device.CreatePixelShader(
                shader_blob.GetBufferPointer(),
                shader_blob.GetBufferSize(),
                None,
            )?
        };

        Ok(Shader::Pixel(shader, shader_blob))
    }

    pub fn vertex_shader(backend: &Backend, path: &str, entry_point: &str) -> Result<Shader> {
        let shader_blob = compile_shader(path, entry_point, "vs_5_0")?;

        let shader = unsafe {
            backend.device.CreateVertexShader(
                shader_blob.GetBufferPointer(),
                shader_blob.GetBufferSize(),
                None,
            )?
        };

        Ok(Shader::Vertex(shader, shader_blob))
    }
}
