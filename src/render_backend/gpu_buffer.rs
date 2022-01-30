use windows::core::*;
use windows::Win32::Graphics::Direct3D11::*;

use super::backend::Backend;

#[derive(Clone)]
pub struct GPUBuffer {
    pub desc: D3D11_BUFFER_DESC,
    pub buffer: ID3D11Buffer,
}

pub struct MappedGPUBuffer<'a, 'b> {
    subresource: D3D11_MAPPED_SUBRESOURCE,
    size_bytes: usize,
    buffer: &'a ID3D11Buffer,
    device_context: &'b ID3D11DeviceContext,
}

impl<'a, 'b> GPUBuffer {
    pub fn new(backend: &Backend, desc: D3D11_BUFFER_DESC) -> Result<GPUBuffer> {
        let buffer = unsafe { backend.device.CreateBuffer(&desc, std::ptr::null())? };
        Ok(GPUBuffer { desc, buffer })
    }

    pub fn vertex_buffer(backend: &Backend, size_bytes: u32) -> Result<GPUBuffer> {
        Self::new(
            backend,
            D3D11_BUFFER_DESC {
                ByteWidth: size_bytes,
                Usage: D3D11_USAGE_DYNAMIC,
                BindFlags: D3D11_BIND_VERTEX_BUFFER,
                CPUAccessFlags: D3D11_CPU_ACCESS_WRITE,
                ..Default::default()
            },
        )
    }

    pub fn index_buffer(backend: &Backend, size_bytes: u32) -> Result<GPUBuffer> {
        Self::new(
            backend,
            D3D11_BUFFER_DESC {
                ByteWidth: size_bytes,
                Usage: D3D11_USAGE_DYNAMIC,
                BindFlags: D3D11_BIND_INDEX_BUFFER,
                CPUAccessFlags: D3D11_CPU_ACCESS_WRITE,
                ..Default::default()
            },
        )
    }

    pub fn constant_buffer(backend: &Backend, size_bytes: u32) -> Result<GPUBuffer> {
        let size_bytes = if size_bytes % 16 != 0 {
            16 * ((size_bytes / 16) + 1)
        } else {
            size_bytes
        };

        Self::new(
            backend,
            D3D11_BUFFER_DESC {
                ByteWidth: size_bytes,
                Usage: D3D11_USAGE_DYNAMIC,
                BindFlags: D3D11_BIND_CONSTANT_BUFFER,
                CPUAccessFlags: D3D11_CPU_ACCESS_WRITE,
                ..Default::default()
            },
        )
    }

    pub fn structured_buffer<T: Sized>(
        backend: &Backend,
        num_elements: u32,
        gpu_write: bool,
    ) -> Result<GPUBuffer> {
        Self::new(
            backend,
            D3D11_BUFFER_DESC {
                ByteWidth: std::mem::size_of::<T>() as u32 * num_elements,
                Usage: if gpu_write {
                    Default::default()
                } else {
                    D3D11_USAGE_DYNAMIC
                },
                CPUAccessFlags: D3D11_CPU_ACCESS_WRITE,
                BindFlags: if gpu_write {
                    D3D11_BIND_UNORDERED_ACCESS | D3D11_BIND_SHADER_RESOURCE
                } else {
                    D3D11_BIND_SHADER_RESOURCE
                },
                MiscFlags: D3D11_RESOURCE_MISC_BUFFER_STRUCTURED,
                StructureByteStride: std::mem::size_of::<T>() as u32,
                ..Default::default()
            },
        )
    }

    pub fn map(&'a self, backend: &'b Backend) -> Result<MappedGPUBuffer<'a, 'b>> {
        let mapped = unsafe {
            backend
                .device_context
                .Map(&self.buffer, 0, D3D11_MAP_WRITE_DISCARD, 0)?
        };

        Ok(MappedGPUBuffer {
            subresource: mapped,
            size_bytes: self.desc.ByteWidth as usize,
            buffer: &self.buffer,
            device_context: &backend.device_context,
        })
    }
}

impl<'a, 'b> MappedGPUBuffer<'a, 'b> {
    pub fn copy_from<T>(&self, data: &[T])
    where
        T: Sized,
    {
        debug_assert!(data.len() * std::mem::size_of::<T>() <= self.size_bytes);

        unsafe {
            self.subresource
                .pData
                .copy_from(data.as_ptr() as _, data.len() * std::mem::size_of::<T>());
        }
    }
}

impl<'a, 'b> Drop for MappedGPUBuffer<'a, 'b> {
    fn drop(&mut self) {
        unsafe {
            self.device_context.Unmap(self.buffer, 0);
        }
    }
}
