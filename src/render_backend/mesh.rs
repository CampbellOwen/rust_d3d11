use glam::{Vec2, Vec3};
use obj::*;
use windows::core::*;

use super::{backend::Backend, gpu_buffer::GPUBuffer};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[repr(C)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
}

#[derive(Clone)]
pub struct GpuMesh {
    pub vertex_buffer: GPUBuffer,
    pub index_buffer: GPUBuffer,
    pub num_indices: u32,
}

#[derive(Debug, Default)]
pub struct CpuMesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl CpuMesh {
    pub fn from_obj(file_name: &str) -> Result<Vec<CpuMesh>> {
        let obj =
            Obj::load(file_name).map_err(|_| Error::fast_error(HRESULT::from_win32(0x80070057)))?;

        let mut meshes = Vec::new();

        for object in obj.data.objects {
            let mut vertices = Vec::new();
            let mut indices = Vec::new();

            let mut index = 0;

            for group in object.groups {
                for poly in group.polys {
                    assert!(poly.0.len() == 3);

                    for index_tuple in poly.0 {
                        let position = Vec3::from(obj.data.position[index_tuple.0]);

                        let uv_index = index_tuple.1.expect("Should have a UV index");
                        let uv = Vec2::from(obj.data.texture[uv_index]);

                        let normal_index = index_tuple.2.expect("Should have a normal index");
                        let normal = Vec3::from(obj.data.normal[normal_index]);

                        vertices.push(Vertex {
                            position,
                            uv,
                            normal,
                        })
                    }
                    indices.push(index);
                    indices.push(index + 1);
                    indices.push(index + 2);

                    index = index + 3
                }
            }

            meshes.push(CpuMesh { vertices, indices });
        }

        Ok(meshes)
    }

    pub fn upload(&self, backend: &Backend) -> Result<GpuMesh> {
        let vertex_buffer_size = self.vertices.len() * std::mem::size_of::<Vertex>();
        let index_buffer_size = self.indices.len() * std::mem::size_of::<u32>();

        let vertex_buffer = GPUBuffer::vertex_buffer(backend, vertex_buffer_size as u32)?;
        let index_buffer = GPUBuffer::index_buffer(backend, index_buffer_size as u32)?;
        {
            let mapped_vertex_buffer = vertex_buffer.map(backend)?;
            let mapped_index_buffer = index_buffer.map(backend)?;

            mapped_vertex_buffer.copy_from(self.vertices.as_slice());
            mapped_index_buffer.copy_from(self.indices.as_slice());
        }

        Ok(GpuMesh {
            index_buffer,
            vertex_buffer,
            num_indices: self.indices.len() as u32,
        })
    }
}

impl GpuMesh {
    //    pub fn from_meshes(
    //        backend: &Backend,
    //        meshes: &[CpuMesh],
    //    ) -> Result<(Vec<GpuMesh>, GPUBuffer, GPUBuffer)> {
    //        let num_vertices = meshes
    //            .iter()
    //            .fold(0, |total, mesh| total + mesh.vertices.len());
    //        let vertex_buffer_size = num_vertices * std::mem::size_of::<Vertex>();
    //
    //        let num_indices = meshes
    //            .iter()
    //            .fold(0, |total, mesh| total + mesh.indices.len());
    //        let index_buffer_size = num_indices * std::mem::size_of::<u32>();
    //
    //        let vertex_buffer = GPUBuffer::vertex_buffer(backend, vertex_buffer_size as u32)?;
    //        let index_buffer = GPUBuffer::index_buffer(backend, index_buffer_size as u32)?;
    //
    //        let mut vertex_staging = Vec::with_capacity(num_vertices);
    //        let mut index_staging = Vec::with_capacity(num_indices);
    //        let mut gpu_meshes = Vec::with_capacity(meshes.len());
    //
    //        let mut vertex_offset = 0;
    //        for mesh in meshes {
    //            mesh.vertices
    //                .iter()
    //                .for_each(|&vertex| vertex_staging.push(vertex));
    //
    //            mesh.indices
    //                .iter()
    //                .for_each(|&index| index_staging.push(index + vertex_offset));
    //
    //            gpu_meshes.push(GpuMesh {
    //                num_indices: mesh.indices.len() as _,
    //                index_offset: vertex_offset,
    //            });
    //
    //            vertex_offset += mesh.vertices.len() as u32;
    //        }
    //
    //        {
    //            let mapped_vertex_buffer = vertex_buffer.map(backend)?;
    //            let mapped_index_buffer = index_buffer.map(backend)?;
    //
    //            mapped_vertex_buffer.copy_from(vertex_staging.as_slice());
    //            mapped_index_buffer.copy_from(index_staging.as_slice());
    //        }
    //
    //        Ok((gpu_meshes, vertex_buffer, index_buffer))
    //    }
}
