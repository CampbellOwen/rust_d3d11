use windows::core::{Error, Result};
use windows::Win32::Graphics::Direct3D11::*;

use super::backend::Backend;
use super::mesh::GpuMesh;
use super::shader::Shader;

#[derive(Default, Clone)]
pub struct DepthAttachment {
    bind_depth_buffer: bool,
    depth_state: Option<ID3D11DepthStencilState>,
    depth_view: Option<ID3D11DepthStencilView>,
}

#[derive(Default)]
pub struct RenderPass {
    depth_attachment: DepthAttachment,
    input_desc: Option<ID3D11InputLayout>,
    pub vertex_stride: u32,
    pub shader_resources: Vec<ID3D11ShaderResourceView>,
    render_targets: Vec<ID3D11RenderTargetView>,
    sampler_states: Vec<ID3D11SamplerState>,
    pixel_shader: Option<Shader>,
    vertex_shader: Option<Shader>,
    execution: Option<Box<dyn Fn(&Self, &Backend, u32) -> Result<()>>>,
    clear_rtv: bool,
}

impl RenderPass {
    pub fn new() -> RenderPass {
        Default::default()
    }

    pub fn enable_depth(mut self, enable_depth: bool) -> Self {
        self.depth_attachment.bind_depth_buffer = enable_depth;
        self
    }

    pub fn depth_state(mut self, depth_state: ID3D11DepthStencilState) -> Self {
        self.depth_attachment.depth_state = Some(depth_state);
        self
    }

    pub fn depth_stencil_view(mut self, depth_stencil_view: ID3D11DepthStencilView) -> Self {
        self.depth_attachment.depth_view = Some(depth_stencil_view);

        self
    }

    pub fn shader_resource(mut self, srv: ID3D11ShaderResourceView) -> Self {
        self.shader_resources.push(srv);

        self
    }

    pub fn render_target(mut self, rtv: ID3D11RenderTargetView) -> Self {
        self.render_targets.push(rtv);

        self
    }

    pub fn sampler_state(mut self, sample_state: ID3D11SamplerState) -> Self {
        self.sampler_states.push(sample_state);

        self
    }

    pub fn clear_rtv(mut self, clear_rtv: bool) -> Self {
        self.clear_rtv = clear_rtv;

        self
    }

    pub fn vertex_shader(
        mut self,
        backend: &Backend,
        shader: Shader,
        input_element_desc: &[D3D11_INPUT_ELEMENT_DESC],
        vertex_stride: u32,
    ) -> Self {
        if let Shader::Vertex(_, _) = shader {
            self.vertex_shader = Some(shader);

            if let Some(Shader::Vertex(_, blob)) = &self.vertex_shader {
                let input_layout = unsafe {
                    backend
                        .device
                        .CreateInputLayout(
                            input_element_desc.as_ptr(),
                            input_element_desc.len() as u32,
                            blob.GetBufferPointer(),
                            blob.GetBufferSize(),
                        )
                        .expect("Create input layout for vertex shader")
                };
                self.input_desc = Some(input_layout);
                self.vertex_stride = vertex_stride;
            }
        } else {
            panic!("Attaching a non-vertex shader to the vertex shader slot");
        }
        self
    }

    pub fn pixel_shader(mut self, shader: Shader) -> Self {
        if let Shader::Pixel(_, _) = shader {
            self.pixel_shader = Some(shader);
        } else {
            panic!("Attaching a non-pixel shader to the pixel shader slot");
        }
        self
    }

    pub fn execution(mut self, func: Box<dyn Fn(&Self, &Backend, u32) -> Result<()>>) -> Self {
        self.execution = Some(func);

        self
    }

    pub fn clear(&self, backend: &Backend) -> Result<()> {
        for rtv in &self.render_targets {
            backend.clear_render_target_view(rtv, [0.0, 0.0, 0.0, 1.0]);
        }

        if self.depth_attachment.bind_depth_buffer {
            if let Some(depth) = &self.depth_attachment.depth_view {
                backend.clear_depth_stencil_view(depth);
            }
        }

        Ok(())
    }

    pub fn bind(&self, backend: &Backend) -> Result<()> {
        let depth_attachment = if self.depth_attachment.bind_depth_buffer {
            &self.depth_attachment.depth_view
        } else {
            &None
        };

        backend.set_render_targets(&[], &None);

        if self.depth_attachment.bind_depth_buffer {
            if let Some(depth_state) = &self.depth_attachment.depth_state {
                unsafe {
                    backend
                        .device_context
                        .OMSetDepthStencilState(depth_state, 0xFF);
                }
            }
        }

        if let Some(Shader::Pixel(s, _)) = &self.pixel_shader {
            unsafe {
                backend.device_context.PSSetShader(s, std::ptr::null(), 0);
            }
        }
        if let Some(Shader::Vertex(s, _)) = &self.vertex_shader {
            unsafe {
                backend.device_context.VSSetShader(s, std::ptr::null(), 0);
            }
        }

        backend.set_sampler_states(self.sampler_states.as_slice());

        backend.set_render_targets(self.render_targets.as_slice(), depth_attachment);

        backend.set_pixel_shader_attachments(&self.shader_resources, 0);
        backend.set_vertex_shader_attachments(&self.shader_resources, 0);

        if let Some(layout) = &self.input_desc {
            unsafe {
                backend.device_context.IASetInputLayout(layout);
            }
        }

        Ok(())
    }

    pub fn execute(&self, backend: &Backend, num_vertices: u32) -> Result<()> {
        debug_assert!(self.execution.is_some());
        if let Some(func) = &self.execution {
            self.bind(backend)?;

            if self.clear_rtv {
                self.clear(backend)?;
            }

            return func(self, backend, num_vertices);
        } else {
            Err(Error::fast_error(windows::core::HRESULT::from_win32(
                0x80004005,
            )))
        }
    }
}
