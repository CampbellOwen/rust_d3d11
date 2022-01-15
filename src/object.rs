use glam::Mat4;
use windows::Win32::Graphics::Direct3D11::*;

use crate::render_backend::{
    backend::{Backend, OBJECT_CONSTANTS},
    gpu_buffer::GPUBuffer,
    mesh::GpuMesh,
};

//trait Obj {
//    const material_id: Vec<u32>;
//}

//struct Object {
//    material_ids: Vec<u32>,
//
//    mesh: Option<GpuMesh>,
//    textures: Vec<ID3D11Resource>,
//    transform: Option<Mat4>,
//}

//impl Object {
//    fn update(&mut self) {}
//
//    fn bind(&self, texture_slot: u32) {}
//}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Flag {
    Opaque,
    Transparent,
    Light,
}

pub trait GameObject {
    fn update(&mut self);
    fn bind(&mut self, backend: &Backend);

    fn material_id(&self) -> Option<u32> {
        None
    }

    fn textures(&self) -> Vec<ID3D11Resource> {
        vec![]
    }

    fn mesh(&self) -> Option<GpuMesh> {
        None
    }

    fn flags(&self) -> Vec<Flag> {
        vec![]
    }
}

pub struct SimpleMesh {
    mesh: GpuMesh,
    transform: Mat4,
    model_cbuffer: GPUBuffer,
    transform_dirty: bool,

    textures: Vec<ID3D11Resource>,
    flags: Vec<Flag>,
}

impl SimpleMesh {
    pub fn new(backend: &Backend, mesh: GpuMesh) -> SimpleMesh {
        SimpleMesh {
            mesh,
            transform: Mat4::IDENTITY,
            transform_dirty: true,
            model_cbuffer: GPUBuffer::constant_buffer(backend, std::mem::size_of::<Mat4>() as u32)
                .expect("Creating constant buffer"),
            textures: vec![],
            flags: vec![Flag::Opaque],
        }
    }
}

impl GameObject for SimpleMesh {
    fn update(&mut self) {}

    fn bind(&mut self, backend: &Backend) {
        if self.transform_dirty {
            {
                let mapped_buffer = self.model_cbuffer.map(backend).expect("Mapping cbuffer");
                mapped_buffer.copy_from(&[self.transform]);
            }

            self.transform_dirty = false;
        }

        unsafe {
            backend.device_context.PSSetConstantBuffers(
                OBJECT_CONSTANTS,
                1,
                &Some(self.model_cbuffer.buffer.clone()),
            );
            backend.device_context.VSSetConstantBuffers(
                OBJECT_CONSTANTS,
                1,
                &Some(self.model_cbuffer.buffer.clone()),
            );
        }
    }
}

//pub enum ObjectData {
//    Mesh(Mesh),
//}
//
//impl ObjectData {
//    pub fn update(&mut self) {}
//}
//
//pub trait GameObject {
//    const data: ObjectData;
//    fn update(&mut self);
//}
//
//struct a {}
//
//impl GameObject for a {
//    const data: ObjectData = ObjectData::Mesh(Mesh {
//        flags: vec![],
//        mesh: None,
//        textures: vec![],
//        transform: Mat4::IDENTITY,
//    });
//
//    fn update(&mut self) {
//        match Self::data {
//            ObjectData::Mesh(mut m) => m.flags.push(Flag::Opaque),
//        }
//    }
//}
