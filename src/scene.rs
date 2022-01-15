use std::collections::HashMap;

use crate::camera::Camera;
use crate::object::{Flag, GameObject, SimpleMesh};
use crate::render_backend::backend::Backend;
use crate::render_backend::mesh::CpuMesh;
use crate::render_backend::{render_pass::RenderPass, texture::Tex};

pub struct Scene {
    pub materials: HashMap<u32, RenderPass>, // For objects doing forward rendering directly to backbuffer
    pub objects: Vec<Box<dyn GameObject>>,
    pub camera: Camera,
}

impl Scene {
    pub fn update(&mut self) {
        self.objects.iter_mut().for_each(|obj| obj.update());
    }
}

pub fn create_minecraft_scene(backend: &Backend) -> Scene {
    let world = &CpuMesh::from_obj("F:\\Models\\lost-empire\\lost_empire_triangulated.obj")
        .expect("Load obj")[0];

    let uploaded_world = world.upload(backend).expect("Upload mesh");

    let world_object = SimpleMesh::new(backend, uploaded_world);

    Scene {
        materials: HashMap::new(),
        objects: vec![Box::new(world_object)],
        camera: Camera::new(backend),
    }
}
