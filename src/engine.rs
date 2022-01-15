use crate::render_backend::renderer::BasicRenderer;
use crate::render_backend::{backend::Backend, renderer::Renderer};
use crate::scene::Scene;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Direct3D::*;
use windows::Win32::Graphics::Direct3D11::*;
use windows::Win32::Graphics::Dxgi::Common::*;
use windows::Win32::Graphics::Dxgi::*;

pub struct Engine {
    pub backend: Backend,
    pub renderer: Option<Box<dyn Renderer>>,
    pub scene: Option<Scene>,
}

fn create_backend(hwnd: HWND) -> Backend {
    let swap_chain_desc = DXGI_SWAP_CHAIN_DESC {
        BufferDesc: DXGI_MODE_DESC {
            Format: DXGI_FORMAT_R8G8B8A8_UNORM,
            ..Default::default()
        },
        SampleDesc: DXGI_SAMPLE_DESC {
            Count: 1,
            ..Default::default()
        },
        BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
        BufferCount: 2,
        OutputWindow: hwnd as isize,
        Windowed: true.into(),
        SwapEffect: DXGI_SWAP_EFFECT_FLIP_DISCARD,
        ..Default::default()
    };

    let mut device = None;
    let mut swapchain = None;
    let mut context = None;

    unsafe {
        D3D11CreateDeviceAndSwapChain(
            None,
            D3D_DRIVER_TYPE_HARDWARE,
            None,
            D3D11_CREATE_DEVICE_DEBUG,
            std::ptr::null(),
            0,
            D3D11_SDK_VERSION,
            &swap_chain_desc,
            &mut swapchain,
            &mut device,
            std::ptr::null_mut(),
            &mut context,
        )
        .expect("Error creating device and swapchain");
    };

    let device = device.expect("Device should be created");
    let swapchain = swapchain.expect("Swapchain should be created");
    let context = context.expect("DeviceContext should be created");

    Backend::new(device, context, swapchain)
}

impl Engine {
    pub fn new(hwnd: HWND) -> Engine {
        let backend = create_backend(hwnd);

        Engine {
            backend,
            renderer: None,
            scene: None,
        }
    }

    pub fn update(&mut self) {
        if let Some(scene) = &mut self.scene {
            scene.update();
        }
    }

    pub fn render(&mut self, time: usize, delta_time: usize) {
        if let Some(scene) = &mut self.scene {
            if let Some(renderer) = &self.renderer {
                renderer.render(&self.backend, scene, time, delta_time)
            }
        }
    }

    pub fn add_basic_renderer(mut self, width: u32, height: u32) -> Self {
        self.renderer = Some(Box::new(BasicRenderer::new(&self.backend, width, height)));

        self
    }

    pub fn add_scene(mut self, scene_fn: &dyn (Fn(&Backend) -> Scene)) -> Self {
        self.scene = Some(scene_fn(&self.backend));

        self
    }
}
