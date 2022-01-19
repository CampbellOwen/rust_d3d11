use glam::{Mat4, Vec3};
use windows::Win32::Graphics::{Direct3D11::*, Dxgi::Common::DXGI_FORMAT_R32_UINT};

use crate::render_backend::{
    backend::{Backend, OBJECT_CONSTANTS},
    gpu_buffer::GPUBuffer,
    mesh::{GpuMesh, Vertex},
    texture::Tex2D,
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

    fn textures(&self) -> &[ID3D11ShaderResourceView] {
        &[]
    }

    fn mesh(&self) -> Option<GpuMesh> {
        None
    }

    fn flags(&self) -> Vec<Flag> {
        vec![]
    }
}

pub struct SimpleMesh {
    pub mesh: GpuMesh,
    pub transform: Mat4,
    pub model_cbuffer: GPUBuffer,
    transform_dirty: bool,
    pub textures: Vec<ID3D11ShaderResourceView>,
    pub flags: Vec<Flag>,
}

impl SimpleMesh {
    pub fn new(backend: &Backend, mesh: GpuMesh) -> SimpleMesh {
        SimpleMesh {
            mesh,
            transform: Mat4::from_translation(Vec3::new(0.0, 10.0, 0.0)),
            transform_dirty: true,
            model_cbuffer: GPUBuffer::constant_buffer(backend, std::mem::size_of::<Mat4>() as u32)
                .expect("Creating constant buffer"),
            textures: vec![],
            flags: vec![Flag::Opaque],
        }
    }
}

impl GameObject for SimpleMesh {
    fn update(&mut self) {
        self.transform *= Mat4::from_translation(Vec3::new(0.0, 0.0, -0.01));
        self.transform_dirty = true;
    }
    fn flags(&self) -> Vec<Flag> {
        self.flags.clone()
    }
    fn textures(&self) -> &[ID3D11ShaderResourceView] {
        self.textures.as_slice()
    }

    fn mesh(&self) -> Option<GpuMesh> {
        Some(self.mesh.clone())
    }

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
        unsafe {
            backend.device_context.IASetIndexBuffer(
                self.mesh.index_buffer.buffer.clone(),
                DXGI_FORMAT_R32_UINT,
                0,
            );
        }

        unsafe {
            backend.device_context.PSSetShaderResources(
                0,
                self.textures.len() as u32,
                self.textures.as_ptr() as _,
            );
        }

        unsafe {
            backend.device_context.IASetVertexBuffers(
                0,
                1,
                &Some(self.mesh.vertex_buffer.buffer.clone()),
                [std::mem::size_of::<Vertex>() as u32].as_ptr(),
                [0].as_ptr(),
            )
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
