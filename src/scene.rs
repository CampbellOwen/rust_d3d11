use std::collections::HashMap;

use crate::camera::Camera;
use crate::object::{Flag, GameObject, SimpleMesh};
use crate::render_backend::backend::Backend;
use crate::render_backend::mesh::CpuMesh;
use crate::render_backend::texture::Tex2D;
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
    //let world = &CpuMesh::from_obj("F:\\Models\\lost-empire\\lost_empire_triangulated.obj")
    //    .expect("Load obj")[0];

    //let albedo = Tex2D::from_file(backend, "F:\\Models\\lost-empire\\lost_empire-RGB.png")
    //    .expect("Load albedo texture");

    let world = &CpuMesh::from_obj("F:\\Models\\vokselia_spawn\\vokselia_spawn_triangulated.obj")
        .expect("Load obj")[0];

    let albedo = Tex2D::from_file(backend, "F:\\Models\\vokselia_spawn\\vokselia_spawn.png")
        .expect("Load albedo texture");

    let albedo_srv = backend
        .shader_resource_view(&albedo, None)
        .expect("albedo SRV");

    let uploaded_world = world.upload(backend).expect("Upload mesh");

    let mut world_object = SimpleMesh::new(backend, uploaded_world);

    world_object.textures.push(albedo_srv);

    Scene {
        materials: HashMap::new(),
        objects: vec![Box::new(world_object)],
        camera: Camera::new(backend),
    }
}
