pub(crate) mod axis;
pub(crate) mod bbox;
pub(crate) mod hdr_mipmaps;
pub(crate) mod light;
pub(crate) mod linearize;
pub(crate) mod mesh;
pub(crate) mod outline;
pub(crate) mod pickobject;
pub(crate) mod skybox;
pub(crate) mod transmission;

pub(crate) use axis::AxisPass;
pub(crate) use bbox::BoundingboxPass;
pub(crate) use hdr_mipmaps::BuildMipmapsPass;
pub(crate) use light::LightPass;
pub(crate) use linearize::LinearizePass;
pub(crate) use mesh::MeshPass;
pub(crate) use outline::OutlinePass;
pub(crate) use pickobject::PickObjectPass;
pub(crate) use skybox::SkyboxPass;
pub(crate) use transmission::TransmissionPass;

use crate::assets::asset_manager::AssetManager;
use crate::entities::EntityRawU64;
use crate::input::Input;
use crate::renderer::pipeline_manager::PipelineKind;
use crate::uniform::{LightUniform, MaterialUniform};
use crate::{BoundingBoxComponent, LightComponent};
use crate::{
    GlobalModelComponent, Globals, MeshComponent, renderer::scene_renderer::RenderContext,
    uniform::ModelUniform,
};

pub(crate) use crate::gpu::manager::*;
pub(crate) use legion::query::IntoQuery;
pub(crate) use legion::{Entity, World};
use wgpu::IndexFormat;

pub(crate) use super::renderer::rendergraph::*;

pub(crate) trait RenderPass {
    #[allow(dead_code)]
    fn name(&self) -> &'static str;
    fn reads(&self) -> &[ResourceId];
    fn writes(&self) -> &[ResourceId];

    fn prepare(
        &mut self,
        _asset_mgr: &AssetManager,
        _world: &World,
        _globals: &Globals,
        _selected: Option<Entity>,
        _input: &Input,
        _ctx: &mut RenderContext,
    ) {
    }

    fn execute(
        &mut self,
        _encoder: &mut wgpu::CommandEncoder,
        _ctx: &mut RenderContext,
        _asset_mgr: &AssetManager,
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

    fn prepare(
        &mut self,
        asset_mgr: &AssetManager,
        world: &World,
        globals: &Globals,
        selected: Option<Entity>,
        input: &Input,
        ctx: &mut RenderContext,
    ) {
        impl_render_pass_enum!(
            self, prepare, asset_mgr, world, globals, selected, input, ctx
        )
    }

    fn execute(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        ctx: &mut RenderContext,
        asset_mgr: &AssetManager,
    ) {
        impl_render_pass_enum!(self, execute, encoder, ctx, asset_mgr)
    }
}
