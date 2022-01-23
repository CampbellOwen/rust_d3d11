use crate::object::GameObject;
use crate::render_backend::{backend::*, gpu_buffer::GPUBuffer};
use glam::{Mat4, Vec3};

pub struct Camera {
    transform: Mat4,
    transform_dirty: bool,
    cbuffer: GPUBuffer,
}

impl Camera {
    pub fn new(backend: &Backend) -> Camera {
        let camera_pos = Vec3::new(0.0, 0.5, -1.0);
        let focal_point = Vec3::new(0.0, 0.0, 0.2);

        let view_dir = focal_point - camera_pos;

        let up = view_dir.cross(Vec3::new(1.0, 0.0, 0.0));

        let transform =
            Mat4::perspective_lh(std::f32::consts::PI / 4.0, 800.0 / 600.0, 0.001, 1000.0)
                * Mat4::look_at_lh(camera_pos, focal_point, up);

        let cbuffer = GPUBuffer::constant_buffer(backend, std::mem::size_of::<Mat4>() as u32)
            .expect("Creating camera cbuffer");

        Camera {
            transform,
            transform_dirty: true,
            cbuffer,
        }
    }
}

impl GameObject for Camera {
    fn update(&mut self) {}
    fn bind(&mut self, backend: &Backend) {
        if self.transform_dirty {
            {
                let mapped_cbuffer = self.cbuffer.map(backend).expect("Mapping camera cbuffer");
                mapped_cbuffer.copy_from(&[self.transform])
            }

            self.transform_dirty = false;
        }

        unsafe {
            backend.device_context.PSSetConstantBuffers(
                FRAME_CONSTANTS,
                1,
                &Some(self.cbuffer.buffer.clone()),
            );
            backend.device_context.VSSetConstantBuffers(
                FRAME_CONSTANTS,
                1,
                &Some(self.cbuffer.buffer.clone()),
            );
        }
    }
}
