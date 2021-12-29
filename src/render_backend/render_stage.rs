use windows::core::{Error, Result};
use windows::Win32::Graphics::Direct3D11::{
    ID3D11DepthStencilState, ID3D11RenderTargetView, ID3D11ShaderResourceView,
    D3D11_INPUT_ELEMENT_DESC,
};

use super::backend::ResourceView;

#[derive(Default)]
pub struct DepthAttachment {
    bind_depth_buffer: bool,
    depth_state: Option<ID3D11DepthStencilState>,
}

#[derive(Default)]
pub struct RenderStage<'a> {
    depth_attachment: DepthAttachment,
    input_desc: Vec<D3D11_INPUT_ELEMENT_DESC>,
    shader_resources: Vec<&'a ID3D11ShaderResourceView>,
    render_target_attachments: Vec<&'a ID3D11RenderTargetView>,
    // pixel shader
    // vertex shader
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

    pub fn input_desc(mut self, input_element_desc: D3D11_INPUT_ELEMENT_DESC) -> Self {
        self.input_desc.push(input_element_desc);
        self
    }

    pub fn shader_resource(mut self, srv: &'a ResourceView) -> Self {
        if let ResourceView::ShaderResourceView(srv) = srv {
            self.shader_resources.push(srv);
        }
        self
    }

    pub fn render_target_attachment(mut self, rtv: &'a ResourceView) -> Self {
        if let ResourceView::RenderTargetView(rtv) = rtv {
            self.render_target_attachments.push(rtv);
        }

        self
    }

    pub fn bind(&self) -> Result<()> {
        todo!()
    }
}
