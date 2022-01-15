use windows::core::Result;
use windows::Win32::Graphics::{Direct3D11::*, Dxgi::IDXGISwapChain};

use super::texture::{Tex, Tex2D};

pub struct Backend {
    pub device: ID3D11Device,
    pub device_context: ID3D11DeviceContext,
    pub swap_chain: IDXGISwapChain,
}

pub const FRAME_CONSTANTS: u32 = 0;
pub const MATERIAL_CONSTANTS: u32 = 1;
pub const OBJECT_CONSTANTS: u32 = 3;

impl Backend {
    pub fn new(
        device: ID3D11Device,
        device_context: ID3D11DeviceContext,
        swap_chain: IDXGISwapChain,
    ) -> Self {
        Backend {
            device,
            device_context,
            swap_chain,
        }
    }

    pub fn backbuffer(&self, buffer: u32) -> Result<Tex2D> {
        let raw_backbuffer: ID3D11Texture2D = unsafe { self.swap_chain.GetBuffer(buffer)? };
        let mut backbuffer_desc = Default::default();
        unsafe { raw_backbuffer.GetDesc(&mut backbuffer_desc) }

        Ok(Tex2D {
            desc: backbuffer_desc,
            texture: raw_backbuffer,
        })
    }

    pub fn depth_stencil_view<'a>(
        &self,
        texture: &Tex2D,
        desc: Option<D3D11_DEPTH_STENCIL_VIEW_DESC>,
    ) -> Result<ID3D11DepthStencilView> {
        let desc = if let Some(desc) = desc {
            &desc
        } else {
            std::ptr::null()
        };

        let dsv = unsafe {
            self.device
                .CreateDepthStencilView(texture.device_texture(), desc)?
        };

        Ok(dsv)
    }

    pub fn render_target_view<'a>(
        &self,
        texture: &impl Tex<'a>,
        desc: Option<D3D11_RENDER_TARGET_VIEW_DESC>,
    ) -> Result<ID3D11RenderTargetView> {
        let desc = if let Some(desc) = desc {
            &desc
        } else {
            std::ptr::null()
        };

        unsafe {
            self.device
                .CreateRenderTargetView(texture.device_texture(), desc)
        }
    }

    pub fn shader_resource_view<'a>(
        &self,
        texture: &impl Tex<'a>,
        desc: Option<D3D11_SHADER_RESOURCE_VIEW_DESC>,
    ) -> Result<ID3D11ShaderResourceView> {
        let desc = if let Some(desc) = desc {
            &desc
        } else {
            std::ptr::null()
        };

        unsafe {
            self.device
                .CreateShaderResourceView(texture.device_texture(), desc)
        }
    }

    pub fn set_render_targets(
        &self,
        render_targets: &[ID3D11RenderTargetView],
        depth_view: &Option<ID3D11DepthStencilView>,
    ) {
        let rtvs: Vec<Option<ID3D11RenderTargetView>> =
            render_targets.iter().map(|rtv| Some(rtv.clone())).collect();
        unsafe {
            self.device_context
                .OMSetRenderTargets(rtvs.len() as u32, rtvs.as_ptr(), depth_view)
        };
    }

    pub fn clear_render_target_view(&self, rtv: &ID3D11RenderTargetView, clear_colour: [f32; 4]) {
        unsafe {
            self.device_context
                .ClearRenderTargetView(rtv, clear_colour.as_ptr())
        }
    }

    pub fn clear_depth_stencil_view(&self, dsv: &ID3D11DepthStencilView) {
        unsafe {
            self.device_context.ClearDepthStencilView(
                dsv,
                (D3D11_CLEAR_DEPTH | D3D11_CLEAR_STENCIL) as u32,
                1.0,
                0x00,
            );
        }
    }

    pub fn set_pixel_shader_attachments(
        &self,
        attachments: &[ID3D11ShaderResourceView],
        start_slot: u32,
    ) {
        let srvs: Vec<Option<ID3D11ShaderResourceView>> =
            attachments.iter().map(|srv| Some(srv.clone())).collect();

        unsafe {
            self.device_context
                .PSSetShaderResources(start_slot, srvs.len() as u32, srvs.as_ptr());
        }
    }

    pub fn set_vertex_shader_attachments(
        &self,
        attachments: &[ID3D11ShaderResourceView],
        start_slot: u32,
    ) {
        let srvs: Vec<Option<ID3D11ShaderResourceView>> =
            attachments.iter().map(|srv| Some(srv.clone())).collect();

        unsafe {
            self.device_context
                .VSSetShaderResources(start_slot, srvs.len() as u32, srvs.as_ptr());
        }
    }

    pub fn unbind_shader_resources(&self) {
        let num = D3D11_COMMONSHADER_INPUT_RESOURCE_SLOT_COUNT;

        let mut srvs = Vec::with_capacity(num as usize);
        for _ in 0..num {
            srvs.push(None);
        }

        unsafe {
            self.device_context
                .PSSetShaderResources(0, num, srvs.as_ptr());
            self.device_context
                .VSSetShaderResources(0, num, srvs.as_ptr());
        }
    }

    pub fn set_sampler_states(&self, sampler_states: &[ID3D11SamplerState]) {
        let samplers: Vec<Option<ID3D11SamplerState>> = sampler_states
            .iter()
            .map(|state| Some(state.clone()))
            .collect();

        unsafe {
            self.device_context
                .PSSetSamplers(0, samplers.len() as u32, samplers.as_ptr());
            self.device_context
                .VSSetSamplers(0, samplers.len() as u32, samplers.as_ptr());
        }
    }
}
