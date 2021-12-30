use windows::core::Result;
use windows::Win32::Graphics::Direct3D11::{
    ID3D11DepthStencilState, ID3D11InputLayout, D3D11_INPUT_ELEMENT_DESC,
};

use super::backend::Backend;
use super::backend::ResourceView;
use super::shader::Shader;

#[derive(Default)]
pub struct DepthAttachment<'a> {
    bind_depth_buffer: bool,
    depth_state: Option<ID3D11DepthStencilState>,
    depth_view: Option<&'a ResourceView>,
}

#[derive(Default)]
pub struct RenderStage<'a> {
    depth_attachment: DepthAttachment<'a>,
    input_desc: Option<ID3D11InputLayout>,
    shader_resources: Vec<&'a ResourceView>,
    render_target_attachments: Vec<&'a ResourceView>,
    pixel_shader: Option<Shader>,
    vertex_shader: Option<Shader>,
    // compute????
}

impl<'a> RenderStage<'a> {
    pub fn new() -> RenderStage<'a> {
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

    pub fn depth_stencil_view(mut self, depth_stencil_view: &'a ResourceView) -> Self {
        if let ResourceView::DepthStencilView(_) = depth_stencil_view {
            self.depth_attachment.depth_view = Some(depth_stencil_view);
        } else {
            panic!("Attaching a non DepthStencilView as a shader resource attachment");
        }

        self
    }

    pub fn shader_resource(mut self, srv: &'a ResourceView) -> Self {
        if let ResourceView::ShaderResourceView(_) = srv {
            self.shader_resources.push(srv);
        } else {
            panic!("Attaching a non ShaderResourceView as a shader resource attachment");
        }

        self
    }

    pub fn render_target_attachment(mut self, rtv: &'a ResourceView) -> Self {
        if let ResourceView::RenderTargetView(_) = rtv {
            self.render_target_attachments.push(rtv);
        } else {
            panic!("Attaching a non RenderTargetView as a render target attachment");
        }

        self
    }

    pub fn vertex_shader(
        mut self,
        backend: &Backend,
        shader: Shader,
        input_element_desc: &[D3D11_INPUT_ELEMENT_DESC],
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

    pub fn clear(&self, backend: &Backend) -> Result<()> {
        for &rtv in &self.render_target_attachments {
            backend.clear_render_target_view(rtv, [0.0, 0.0, 0.0, 1.0])?;
        }

        if self.depth_attachment.bind_depth_buffer {
            if let Some(depth) = self.depth_attachment.depth_view {
                backend.clear_depth_stencil_view(depth)?;
            }
        }

        Ok(())
    }

    pub fn bind(&self, backend: &Backend) -> Result<()> {
        let depth_attachment = if self.depth_attachment.bind_depth_buffer {
            self.depth_attachment.depth_view
        } else {
            None
        };

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

        backend.set_render_targets(self.render_target_attachments.as_slice(), depth_attachment);

        backend
            .set_pixel_shader_attachments(&self.shader_resources, 0)
            .expect("Setting pixel shader attachments");
        backend
            .set_vertex_shader_attachments(&self.shader_resources, 0)
            .expect("Setting pixel shader attachments");

        if let Some(layout) = &self.input_desc {
            unsafe {
                backend.device_context.IASetInputLayout(layout);
            }
        }

        //unsafe {backend.device_context.OMSetRenderTargets(self.render_target_attachments.len() as u32, self.render_target_attachments.as_ptr(), depth_attachment);}

        //self.shader_resources

        Ok(())
    }
}
