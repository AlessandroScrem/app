pub(crate) mod axis;
pub(crate) mod bbox;
pub(crate) mod build_mipmaps;
pub(crate) mod light;
pub(crate) mod linearize;
pub(crate) mod mesh;
pub(crate) mod outline;
pub(crate) mod pickobject;
pub(crate) mod skybox;

pub(crate) use axis::*;
pub(crate) use bbox::*;
pub(crate) use build_mipmaps::*;
pub(crate) use light::*;
pub(crate) use linearize::*;
pub(crate) use mesh::*;
pub(crate) use outline::*;
pub(crate) use pickobject::*;
pub(crate) use skybox::*;

use crate::renderer::FrameData;
use crate::gpu::pipeline_manager::PipelineKind;
use crate::renderer::scene_renderer::RenderContext;

pub(crate) use crate::gpu::caches::*;
use wgpu::IndexFormat;

use super::rendergraph::ResourceId;
use crate::assets::MaterialId;


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
    Transmission(MeshPass),
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
            Self::Transmission(p) => p.$method($($arg),*),
            Self::Skybox(p) => p.$method($($arg),*),
            Self::BuildMipmaps(p) => p.$method($($arg),*),
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
