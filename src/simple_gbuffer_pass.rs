use crate::render_backend::{backend::Backend, render_pass::RenderPass, shader::Shader};
use windows::Win32::{Foundation::*, Graphics::Direct3D11::*, Graphics::Dxgi::Common::*};

//pub fn create_gbuffer_pass(
//    backend: &Backend,
//    position: ID3D11RenderTargetView,
//    albedo: ID3D11RenderTargetView,
//    normal: ID3D11RenderTargetView,
//) -> RenderPass {
//    let input_desc = [
//        D3D11_INPUT_ELEMENT_DESC {
//            SemanticName: PSTR(b"POSITION\0".as_ptr() as _),
//            SemanticIndex: 0,
//            Format: DXGI_FORMAT_R32G32B32_FLOAT,
//            InputSlot: 0,
//            AlignedByteOffset: 0,
//            InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
//            InstanceDataStepRate: 0,
//        },
//        D3D11_INPUT_ELEMENT_DESC {
//            SemanticName: PSTR(b"COLOR\0".as_ptr() as _),
//            SemanticIndex: 0,
//            Format: DXGI_FORMAT_R32G32B32A32_FLOAT,
//            InputSlot: 0,
//            AlignedByteOffset: 12,
//            InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
//            InstanceDataStepRate: 0,
//        },
//    ];
//
//    let depth_stencil_desc = D3D11_DEPTH_STENCIL_DESC {
//        DepthEnable: true.into(),
//        DepthWriteMask: D3D11_DEPTH_WRITE_MASK_ALL,
//        DepthFunc: D3D11_COMPARISON_LESS,
//        StencilEnable: true.into(),
//        StencilReadMask: 0xFF,
//        StencilWriteMask: 0xFF,
//        FrontFace: D3D11_DEPTH_STENCILOP_DESC {
//            StencilFailOp: D3D11_STENCIL_OP_KEEP,
//            StencilDepthFailOp: D3D11_STENCIL_OP_INCR,
//            StencilPassOp: D3D11_STENCIL_OP_KEEP,
//            StencilFunc: D3D11_COMPARISON_ALWAYS,
//        },
//        BackFace: D3D11_DEPTH_STENCILOP_DESC {
//            StencilFailOp: D3D11_STENCIL_OP_KEEP,
//            StencilDepthFailOp: D3D11_STENCIL_OP_DECR,
//            StencilPassOp: D3D11_STENCIL_OP_KEEP,
//            StencilFunc: D3D11_COMPARISON_ALWAYS,
//        },
//    };
//
//    let depth_stencil_state = unsafe {
//        backend
//            .device
//            .CreateDepthStencilState(&depth_stencil_desc)
//            .expect("Create depth stencil state")
//    };
//
//    RenderPass::new()
//        .enable_depth(true)
//        .depth_state(depth_stencil_state)
//        .render_target(position)
//        .render_target(albedo)
//        .render_target(normal)
//        .vertex_shader(
//            backend,
//            Shader::vertex_shader(backend, "gbuffer.hlsl", "vertex").expect("Create vertex shader"),
//            &input_desc,
//            std::mem::size_of::<Vertex>() as u32,
//        )
//        .pixel_shader(
//            Shader::pixel_shader(backend, "gbuffer.hlsl", "pixel").expect("Create pixel shader"),
//        )
//}
