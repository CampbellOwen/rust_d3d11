use windows::core::*;
use windows::Win32::Graphics::Direct3D11::*;

use super::backend::Backend;
pub struct GPUBuffer {
    pub desc: D3D11_BUFFER_DESC,
    pub buffer: ID3D11Buffer,
}

pub struct MappedGPUBuffer<'a, 'b> {
    subresource: D3D11_MAPPED_SUBRESOURCE,
    buffer: &'a ID3D11Buffer,
    device_context: &'b ID3D11DeviceContext,
}

impl<'a, 'b> GPUBuffer {
    pub fn new(backend: &Backend, desc: D3D11_BUFFER_DESC) -> Result<GPUBuffer> {
        let buffer = unsafe { backend.device.CreateBuffer(&desc, std::ptr::null())? };
        Ok(GPUBuffer { desc, buffer })
    }

    pub fn map(&'a self, backend: &'b Backend) -> Result<MappedGPUBuffer<'a, 'b>> {
        let mapped = unsafe {
            backend
                .device_context
                .Map(&self.buffer, 0, D3D11_MAP_WRITE_DISCARD, 0)?
        };

        Ok(MappedGPUBuffer {
            subresource: mapped,
            buffer: &self.buffer,
            device_context: &backend.device_context,
        })
    }
}

impl<'a, 'b> MappedGPUBuffer<'a, 'b> {
    pub fn copy_from<T>(&self, data: *const T, num_bytes: usize) {
        unsafe {
            self.subresource.pData.copy_from(data as _, num_bytes);
        };
    }
}

impl<'a, 'b> Drop for MappedGPUBuffer<'a, 'b> {
    fn drop(&mut self) {
        unsafe {
            self.device_context.Unmap(self.buffer, 0);
        }
    }
}
