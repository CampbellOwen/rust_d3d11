use glam::{Vec2, Vec3};
use windows::{core::*, Win32::Graphics::Direct3D11::*};

use super::{backend::Backend, gpu_buffer::GPUBuffer};

#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
}

#[derive(Debug, Default)]
pub struct CpuMesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct GpuMesh {
    pub num_indices: u32,
    pub index_offset: u32,
}

pub fn upload_meshes(
    backend: &Backend,
    meshes: &[CpuMesh],
) -> Result<(Vec<GpuMesh>, GPUBuffer, GPUBuffer)> {
    let num_vertices = meshes
        .iter()
        .fold(0, |total, mesh| total + mesh.vertices.len());
    let vertex_buffer_size = num_vertices * std::mem::size_of::<Vertex>();

    let num_indices = meshes
        .iter()
        .fold(0, |total, mesh| total + mesh.indices.len());
    let index_buffer_size = num_indices * std::mem::size_of::<u32>();

    let vertex_buffer = GPUBuffer::new(
        backend,
        D3D11_BUFFER_DESC {
            ByteWidth: vertex_buffer_size as u32,
            Usage: D3D11_USAGE_DYNAMIC,
            BindFlags: D3D11_BIND_VERTEX_BUFFER,
            CPUAccessFlags: D3D11_CPU_ACCESS_WRITE,
            ..Default::default()
        },
    )?;

    let index_buffer = GPUBuffer::new(
        backend,
        D3D11_BUFFER_DESC {
            ByteWidth: index_buffer_size as u32,
            Usage: D3D11_USAGE_DYNAMIC,
            BindFlags: D3D11_BIND_INDEX_BUFFER,
            CPUAccessFlags: D3D11_CPU_ACCESS_WRITE,
            ..Default::default()
        },
    )?;

    let mut vertex_staging = Vec::with_capacity(num_vertices);
    let mut index_staging = Vec::with_capacity(num_indices);
    let mut gpu_meshes = Vec::with_capacity(meshes.len());

    let mut vertex_offset = 0;
    for mesh in meshes {
        mesh.vertices
            .iter()
            .for_each(|&vertex| vertex_staging.push(vertex));

        mesh.indices
            .iter()
            .for_each(|&index| index_staging.push(index + vertex_offset));

        gpu_meshes.push(GpuMesh {
            num_indices: mesh.indices.len() as _,
            index_offset: vertex_offset,
        });

        vertex_offset += mesh.vertices.len() as u32;
    }

    {
        let mapped_vertex_buffer = vertex_buffer.map(backend)?;
        let mapped_index_buffer = index_buffer.map(backend)?;

        mapped_vertex_buffer.copy_from(vertex_staging.as_slice());
        mapped_index_buffer.copy_from(index_staging.as_slice());
    }

    Ok((gpu_meshes, vertex_buffer, index_buffer))
}
