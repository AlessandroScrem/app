pub(crate) mod axis;
pub(crate) mod bbox;
pub(crate) mod build_mipmaps;
pub(crate) mod light;
pub(crate) mod linearize;
pub(crate) mod mesh;
pub(crate) mod outline;
pub(crate) mod pickobject;
pub(crate) mod skybox;
pub(crate) mod transmission;

pub(crate) use axis::AxisPass;
pub(crate) use bbox::BoundingboxPass;
pub(crate) use build_mipmaps::BuildMipmapsPass;
pub(crate) use light::LightPass;
pub(crate) use linearize::LinearizePass;
pub(crate) use mesh::MeshPass;
pub(crate) use outline::OutlinePass;
pub(crate) use pickobject::PickObjectPass;
pub(crate) use skybox::SkyboxPass;
pub(crate) use transmission::TransmissionPass;

use crate::renderer::FrameData;
use crate::renderer::pipeline_manager::PipelineKind;
use crate::renderer::scene_renderer::RenderContext;

pub(crate) use crate::gpu::manager::*;
use wgpu::IndexFormat;

pub(crate) use super::renderer::rendergraph::*;

// resources needeed
// mesh :
//      gpu_mesh: &'a GpuMesh,
//      gpu_manager = ctx.gpu_mgr;
//      material_bg: &'a wgpu::BindGroup,
//      index_range: &'a std::ops::Range<u32>,
//      pipeline_manager = ctx.pip_mgr;

// skybox:
//      globals:
//      gpu_manager = ctx.gpu_mgr;
//      pipeline_manager = ctx.pip_mgr;
//      skybox_manager = ctx.skb_mgr;

// build mipmaps:
//      globals:
//      gpu_manager = ctx.gpu_mgr;
//      pipeline_manager = ctx.pip_mgr;
//

// transmission:
//      gpu_mesh: &'a GpuMesh,
//      material_bg: &'a wgpu::BindGroup,
//      index_range: &'a std::ops::Range<u32>,
//      pipeline_manager = ctx.pip_mgr;
//      gpu_manager = ctx.gpu_mgr;
//

// light:
//      pipeline_manager = ctx.pip_mgr;
//      gpu_manager = ctx.gpu_mgr;
//

// axis:
//      globals:
//      pipeline_manager = ctx.pip_mgr;
//      gpu_manager = ctx.gpu_mgr;
//

// bbox:
//      globals:
//      vertexbuffer;
//      count;
//      pipeline_manager = ctx.pip_mgr;
//      gpu_manager = ctx.gpu_mgr;
//

// linearize:
//      pipeline_manager = ctx.pip_mgr;
//      gpu_manager = ctx.gpu_mgr;
//

// outline:
//      selected;
//      pipeline_manager = ctx.pip_mgr;
//      gpu_manager = ctx.gpu_mgr;
//

// pickobject:
//      input;
//      pickobject = ctx.pickobject;
//      gpu_manager = ctx.gpu_mgr;

pub(crate) trait RenderPass {
    #[allow(dead_code)]
    fn name(&self) -> &'static str;
    fn reads(&self) -> &[ResourceId];
    fn writes(&self) -> &[ResourceId];

    fn execute(
        &mut self,
        _encoder: &mut wgpu::CommandEncoder,
        _ctx: &mut RenderContext,
        _frame: &FrameData,
    ) {
    }
}

pub(crate) enum RenderPassEnum {
    Mesh(MeshPass),
    Transmission(TransmissionPass),
    BuildMipmaps(BuildMipmapsPass),
    Light(LightPass),
    Skybox(SkyboxPass),
    Axis(AxisPass),
    BBox(BoundingboxPass),
    Linearize(LinearizePass),
    Outline(OutlinePass),
    PickObject(PickObjectPass),
}

macro_rules! impl_render_pass_enum {
    ($self:ident, $method:ident $(, $arg:ident)*) => {
        match $self {
            Self::Mesh(p) => p.$method($($arg),*),
            Self::Skybox(p) => p.$method($($arg),*),
            Self::BuildMipmaps(p) => p.$method($($arg),*),
            Self::Transmission(p) => p.$method($($arg),*),
            Self::Light(p) => p.$method($($arg),*),
            Self::Axis(p) => p.$method($($arg),*),
            Self::BBox(p) => p.$method($($arg),*),
            Self::Linearize(p) => p.$method($($arg),*),
            Self::Outline(p) => p.$method($($arg),*),
            Self::PickObject(p) => p.$method($($arg),*),
        }
    };
}

impl RenderPass for RenderPassEnum {
    fn name(&self) -> &'static str {
        impl_render_pass_enum!(self, name)
    }

    fn reads(&self) -> &[ResourceId] {
        impl_render_pass_enum!(self, reads)
    }

    fn writes(&self) -> &[ResourceId] {
        impl_render_pass_enum!(self, writes)
    }

    fn execute(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        ctx: &mut RenderContext,
        frame: &FrameData,
    ) {
        impl_render_pass_enum!(self, execute, encoder, ctx, frame)
    }
}
