use windows::core::{Error, Result, HRESULT};
use windows::Win32::Graphics::{Direct3D11::*, Dxgi::IDXGISwapChain};

use super::texture::{CoreTexture, Texture};

pub enum ResourceView {
    RenderTargetView(ID3D11RenderTargetView),
    DepthStencilView(ID3D11DepthStencilView),
}

pub struct Backend {
    pub device: ID3D11Device,
    pub device_context: ID3D11DeviceContext,
    pub swap_chain: IDXGISwapChain,
}

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

    pub fn backbuffer(&self, buffer: u32) -> Result<Texture> {
        let raw_backbuffer: ID3D11Texture2D = unsafe { self.swap_chain.GetBuffer(buffer)? };
        let mut backbuffer_desc = Default::default();
        unsafe { raw_backbuffer.GetDesc(&mut backbuffer_desc) }

        Ok(Texture::from_swapchain(raw_backbuffer, backbuffer_desc))
    }

    pub fn depth_stencil_view(
        &self,
        texture: &Texture,
        desc: Option<D3D11_DEPTH_STENCIL_VIEW_DESC>,
    ) -> Result<ResourceView> {
        if let CoreTexture::Texture2D(raw_texture) = &texture.resource {
            let raw_view = if let Some(desc) = desc {
                unsafe { self.device.CreateDepthStencilView(raw_texture, &desc)? }
            } else {
                unsafe {
                    self.device
                        .CreateDepthStencilView(raw_texture, std::ptr::null())?
                }
            };

            return Ok(ResourceView::DepthStencilView(raw_view));
        }

        Err(Error::fast_error(HRESULT::from_win32(0x80070057))) // E_INVALIDARG
    }

    pub fn render_target_view(
        &self,
        texture: &Texture,
        desc: Option<D3D11_RENDER_TARGET_VIEW_DESC>,
    ) -> Result<ResourceView> {
        let desc = if let Some(desc) = desc {
            &desc
        } else {
            std::ptr::null()
        };

        let raw_view = match &texture.resource {
            CoreTexture::Texture2D(tex) => unsafe {
                self.device.CreateRenderTargetView(tex, desc)?
            },
        };

        Ok(ResourceView::RenderTargetView(raw_view))
    }

    pub fn set_render_targets(
        &self,
        render_targets: &[ResourceView],
        depth_view: Option<&ResourceView>,
    ) {
        let rtvs: Vec<Option<ID3D11RenderTargetView>> = render_targets
            .iter()
            .filter_map(|rtv| {
                if let ResourceView::RenderTargetView(rtv) = rtv {
                    Some(rtv.clone())
                } else {
                    None
                }
            })
            .map(|rtv| Some(rtv))
            .collect();

        let depth_view = if let Some(dsv) = depth_view {
            if let ResourceView::DepthStencilView(dsv) = dsv {
                Some(dsv.clone())
            } else {
                None
            }
        } else {
            None
        };

        unsafe {
            self.device_context.OMSetRenderTargets(
                render_targets.len() as u32,
                rtvs.as_ptr(),
                depth_view,
            )
        };
    }

    pub fn clear_render_target_view(
        &self,
        rtv: &ResourceView,
        clear_colour: [f32; 4],
    ) -> Result<()> {
        if let ResourceView::RenderTargetView(rtv) = rtv {
            unsafe {
                self.device_context
                    .ClearRenderTargetView(rtv, clear_colour.as_ptr())
            }
            Ok(())
        } else {
            Err(Error::fast_error(HRESULT::from_win32(0x80070057))) // E_INVALIDARG
        }
    }

    pub fn clear_depth_stencil_view(&self, dsv: &ResourceView) -> Result<()> {
        if let ResourceView::DepthStencilView(dsv) = dsv {
            unsafe {
                self.device_context.ClearDepthStencilView(
                    dsv,
                    (D3D11_CLEAR_DEPTH | D3D11_CLEAR_STENCIL) as u32,
                    1.0,
                    0x00,
                );
            }
            Ok(())
        } else {
            Err(Error::fast_error(HRESULT::from_win32(0x80070057))) // E_INVALIDARG
        }
    }
}
