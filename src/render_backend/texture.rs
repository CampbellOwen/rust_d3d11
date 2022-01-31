use std::marker::PhantomData;

use image::io::Reader as ImageReader;

use windows::{
    core::*,
    Win32::Graphics::Direct3D11::*,
    Win32::Graphics::Dxgi::{Common::*, DXGI_SWAP_CHAIN_DESC},
};

use super::backend::Backend;

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

    pub fn build_texture3d(&self) -> D3D11_TEXTURE3D_DESC {
        D3D11_TEXTURE3D_DESC {
            Width: self.size[0],
            Height: self.size[1],
            Depth: self.size[2],
            MipLevels: self.mip_levels,
            Format: self.format,
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
impl D3DTextureDesc for D3D11_TEXTURE3D_DESC {}

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
impl<'a> D3DTexture<'a> for ID3D11Texture3D {}

pub trait Tex<'a>: Sized {
    type TextureType: D3DTexture<'a>;
    type DescType: D3DTextureDesc;

    fn device_texture(&self) -> Self::TextureType;
    fn desc(&self) -> Self::DescType;
    fn new(backend: &Backend, desc: Self::DescType) -> Result<Self>;
    fn from_file(backend: &Backend, file: &str) -> Result<Self>;
}

pub struct Tex3D {
    pub texture: ID3D11Texture3D,
    pub desc: D3D11_TEXTURE3D_DESC,
}

impl<'a> Tex<'a> for Tex3D {
    type TextureType = ID3D11Texture3D;

    type DescType = D3D11_TEXTURE3D_DESC;

    fn device_texture(&self) -> Self::TextureType {
        self.texture.clone()
    }

    fn desc(&self) -> Self::DescType {
        self.desc
    }

    fn new(backend: &Backend, desc: Self::DescType) -> Result<Self> {
        let texture = unsafe { backend.device.CreateTexture3D(&desc, std::ptr::null())? };

        Ok(Tex3D { desc, texture })
    }

    fn from_file(backend: &Backend, file: &str) -> Result<Self> {
        todo!()
    }
}

pub struct Tex2D {
    pub texture: ID3D11Texture2D,
    pub desc: D3D11_TEXTURE2D_DESC,
}

impl<'a> Tex<'a> for Tex2D {
    type TextureType = ID3D11Texture2D;
    type DescType = D3D11_TEXTURE2D_DESC;

    fn device_texture(&self) -> ID3D11Texture2D {
        self.texture.clone()
    }

    fn desc(&self) -> D3D11_TEXTURE2D_DESC {
        self.desc
    }

    fn new(backend: &Backend, desc: D3D11_TEXTURE2D_DESC) -> Result<Tex2D> {
        let texture = unsafe { backend.device.CreateTexture2D(&desc, std::ptr::null())? };

        Ok(Tex2D { desc, texture })
    }

    fn from_file(backend: &Backend, file: &str) -> Result<Self> {
        let img = ImageReader::open(file)
            .map_err(|_| Error::fast_error(HRESULT::from_win32(0x80070057)))?
            .decode()
            .map_err(|_| HRESULT::from_win32(0x80070057))?;

        let img = img.to_rgba8();

        let mut samples = img.into_flat_samples();
        let (_, w, h) = samples.extents();
        let (_, width_stride, height_stride) = samples.strides_cwh();

        let desc = TextureDescBuilder::new()
            .size([w as u32, h as u32, 0])
            .mip_levels(0)
            .bind_flags(D3D11_BIND_SHADER_RESOURCE | D3D11_BIND_RENDER_TARGET)
            .misc_flags(D3D11_RESOURCE_MISC_GENERATE_MIPS)
            .cpu_access_flags(D3D11_CPU_ACCESS_WRITE)
            .format(DXGI_FORMAT_R8G8B8A8_UNORM_SRGB)
            .build_texture2d();

        let texture = unsafe {
            backend
                .device
                .CreateTexture2D(&desc, std::ptr::null_mut())?
        };

        unsafe {
            backend.device_context.UpdateSubresource(
                texture.clone(),
                0,
                std::ptr::null(),
                samples.samples.as_mut_ptr() as _,
                height_stride as u32,
                (height_stride * h) as u32,
            );
        }

        let tex = Tex2D { desc, texture };

        let srv = backend.shader_resource_view(&tex, None)?;

        unsafe {
            backend.device_context.GenerateMips(&srv);
        }

        Ok(tex)
    }
}
