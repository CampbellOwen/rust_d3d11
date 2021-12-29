use windows::{
    core::*,
    Win32::Graphics::Direct3D11::*,
    Win32::Graphics::Dxgi::{Common::*, DXGI_SWAP_CHAIN_DESC, DXGI_USAGE_RENDER_TARGET_OUTPUT},
};

use super::backend::{Backend, ResourceView};

#[derive(Clone, Copy, Debug)]
pub enum TextureType {
    Texture2D,
}

#[derive(Clone, Copy)]
pub struct TextureDesc {
    tex_type: TextureType,
    size: [u32; 3],
    mip_levels: u32,
    array_size: u32,
    format: DXGI_FORMAT,
    sample_desc: DXGI_SAMPLE_DESC,
    usage: D3D11_USAGE,
    bind_flags: D3D11_BIND_FLAG,
    cpu_access_flags: D3D11_CPU_ACCESS_FLAG,
    misc_flags: D3D11_RESOURCE_MISC_FLAG,
}

impl TextureDesc {
    pub fn new(tex_type: TextureType) -> TextureDesc {
        TextureDesc {
            tex_type,
            size: Default::default(),
            mip_levels: Default::default(),
            array_size: 1,
            format: Default::default(),
            sample_desc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            usage: Default::default(),
            bind_flags: Default::default(),
            cpu_access_flags: Default::default(),
            misc_flags: Default::default(),
        }
    }
    pub fn new_2d() -> TextureDesc {
        TextureDesc::new(TextureType::Texture2D)
    }

    pub fn size(mut self, size: [u32; 3]) -> Self {
        self.size = size;
        self
    }

    pub fn mip_levels(mut self, mip_levels: u32) -> Self {
        self.mip_levels = mip_levels;
        self
    }

    pub fn array_size(mut self, array_size: u32) -> Self {
        self.array_size = array_size;
        self
    }

    pub fn format(mut self, format: DXGI_FORMAT) -> Self {
        self.format = format;
        self
    }

    pub fn sample_desc(mut self, sample_desc: DXGI_SAMPLE_DESC) -> Self {
        self.sample_desc = sample_desc;
        self
    }

    pub fn usage(mut self, usage: D3D11_USAGE) -> Self {
        self.usage = usage;
        self
    }

    pub fn bind_flags(mut self, bind_flags: D3D11_BIND_FLAG) -> Self {
        self.bind_flags = bind_flags;
        self
    }

    pub fn cpu_access_flags(mut self, cpu_access_flags: D3D11_CPU_ACCESS_FLAG) -> Self {
        self.cpu_access_flags = cpu_access_flags;
        self
    }

    pub fn misc_flags(mut self, misc_flags: D3D11_RESOURCE_MISC_FLAG) -> Self {
        self.misc_flags = misc_flags;
        self
    }
}

impl From<DXGI_SWAP_CHAIN_DESC> for TextureDesc {
    fn from(desc: DXGI_SWAP_CHAIN_DESC) -> Self {
        Self {
            tex_type: TextureType::Texture2D,
            size: [desc.BufferDesc.Width, desc.BufferDesc.Height, 0],
            mip_levels: 1,
            array_size: 1,
            format: desc.BufferDesc.Format,
            sample_desc: desc.SampleDesc,
            usage: D3D11_USAGE_DEFAULT,
            bind_flags: D3D11_BIND_RENDER_TARGET,
            cpu_access_flags: 0,
            misc_flags: 0,
        }
    }
}

impl From<D3D11_TEXTURE2D_DESC> for TextureDesc {
    fn from(desc: D3D11_TEXTURE2D_DESC) -> Self {
        Self {
            tex_type: TextureType::Texture2D,
            size: [desc.Width, desc.Height, 0],
            mip_levels: desc.MipLevels,
            array_size: desc.ArraySize,
            format: desc.Format,
            sample_desc: desc.SampleDesc,
            usage: desc.Usage,
            bind_flags: desc.BindFlags,
            cpu_access_flags: desc.CPUAccessFlags,
            misc_flags: desc.MiscFlags,
        }
    }
}

pub enum CoreTexture {
    Texture2D(ID3D11Texture2D),
}

impl CoreTexture {
    //pub fn resource(&self) -> &ID3D11Resource {
    //    match &self {
    //        &CoreTexture::Texture2D(raw_texture) => raw_texture.into_param::<&ID3D11Resource>(),
    //    }
    //}
}

pub struct Texture {
    pub resource: CoreTexture,
    pub desc: TextureDesc,
}

impl Texture {
    pub fn new(backend: &Backend, desc: TextureDesc) -> Result<Texture> {
        let core_texture = match desc.tex_type {
            TextureType::Texture2D => {
                let core_desc = D3D11_TEXTURE2D_DESC {
                    Width: desc.size[0],
                    Height: desc.size[1],
                    MipLevels: desc.mip_levels,
                    ArraySize: desc.array_size,
                    Format: desc.format,
                    SampleDesc: desc.sample_desc,
                    Usage: desc.usage,
                    BindFlags: desc.bind_flags,
                    CPUAccessFlags: desc.cpu_access_flags,
                    MiscFlags: desc.misc_flags,
                };
                CoreTexture::Texture2D(unsafe {
                    backend
                        .device
                        .CreateTexture2D(&core_desc, std::ptr::null())?
                })
            }
        };

        Ok(Texture {
            desc,
            resource: core_texture,
        })
    }

    pub fn from_swapchain(buffer: ID3D11Texture2D, desc: D3D11_TEXTURE2D_DESC) -> Texture {
        let desc = TextureDesc::from(desc);
        let resource = CoreTexture::Texture2D(buffer);

        Texture { desc, resource }
    }
}
