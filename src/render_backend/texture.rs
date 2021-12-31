use std::marker::PhantomData;

use windows::{
    core::*,
    Win32::Graphics::Direct3D11::*,
    Win32::Graphics::Dxgi::{Common::*, DXGI_SWAP_CHAIN_DESC},
};

use super::backend::Backend;

#[derive(Clone, Copy, Debug)]
pub enum TextureType {
    Texture2D,
}

#[derive(Clone, Copy)]
pub struct TextureDescBuilder {
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

impl TextureDescBuilder {
    pub fn new() -> TextureDescBuilder {
        TextureDescBuilder {
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

    pub fn build_texture2d(&self) -> D3D11_TEXTURE2D_DESC {
        D3D11_TEXTURE2D_DESC {
            Width: self.size[0],
            Height: self.size[1],
            MipLevels: self.mip_levels,
            ArraySize: self.array_size,
            Format: self.format,
            SampleDesc: self.sample_desc,
            Usage: self.usage,
            BindFlags: self.bind_flags,
            CPUAccessFlags: self.cpu_access_flags,
            MiscFlags: self.misc_flags,
        }
    }
}

impl Default for TextureDescBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub trait D3DTextureDesc {}

impl D3DTextureDesc for D3D11_TEXTURE2D_DESC {}

impl From<DXGI_SWAP_CHAIN_DESC> for TextureDescBuilder {
    fn from(desc: DXGI_SWAP_CHAIN_DESC) -> Self {
        Self {
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

impl From<D3D11_TEXTURE2D_DESC> for TextureDescBuilder {
    fn from(desc: D3D11_TEXTURE2D_DESC) -> Self {
        Self {
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

pub trait D3DTexture<'a>: IntoParam<'a, ID3D11Resource> + Clone {}
impl<'a> D3DTexture<'a> for ID3D11Texture2D {}

pub struct Texture<'a, T, D>
where
    T: D3DTexture<'a> + IntoParam<'a, ID3D11Resource>,
    D: D3DTextureDesc,
{
    pub texture: T,
    pub desc: D,
    pub phantom: PhantomData<&'a T>,
}

pub type Texture2D<'a> = Texture<'a, ID3D11Texture2D, D3D11_TEXTURE2D_DESC>;

impl<'a> Texture2D<'a> {
    pub fn new(backend: &Backend, desc: D3D11_TEXTURE2D_DESC) -> Result<Texture2D> {
        let texture = unsafe { backend.device.CreateTexture2D(&desc, std::ptr::null())? };

        Ok(Texture {
            desc,
            texture,
            phantom: PhantomData,
        })
    }
}
