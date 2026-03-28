pub(crate) mod axis;
pub(crate) mod bbox;
pub(crate) mod light;
pub(crate) mod linearize;
pub(crate) mod mesh;
pub(crate) mod outline;
pub(crate) mod pickobject;
pub(crate) mod skybox;

pub(crate) use axis::AxisPass;
pub(crate) use bbox::BBoxPass;
pub(crate) use light::LightPass;
pub(crate) use linearize::LinearizePass;
pub(crate) use mesh::MeshPass;
pub(crate) use outline::OutlinePass;
pub(crate) use pickobject::PickObjectPass;
pub(crate) use skybox::SkyboxPass;

use crate::assets::asset_manager::AssetManager;
use crate::entities::EntityRawU64;
use crate::input::Input;
use crate::renderer::pipeline_manager::PipelineKind;
use crate::uniform::{LightUniform, MaterialUniform};
use crate::{BoundingBoxComponent, LightComponent};
use crate::{GlobalModelComponent, Globals, MeshComponent, renderer::scene_renderer::RenderContext,
    uniform::ModelUniform,
};

pub(crate) use legion::query::IntoQuery;
pub(crate) use legion::{Entity, World};
use wgpu::IndexFormat;
pub (crate) use crate::gpu::manager::*;

pub (crate) use super::renderer::rendergraph::*;

pub(crate) trait RenderPass {
    #[allow(dead_code)]
    fn name(&self) -> &'static str;
    fn reads(&self) -> &[ResourceId];
    fn writes(&self) -> &[ResourceId];

    fn prepare(
        &mut self,
        asset_mgr: &AssetManager,
        world: &World,
        globals: &Globals,
        selected: Option<Entity>,
        input: &Input,
        ctx: &mut RenderContext,
    );

    fn execute(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        ctx: &mut RenderContext,
        asset_mgr: &AssetManager,
    );
}


pub(crate) enum RenderPassEnum {
    Mesh(MeshPass),
    Light(LightPass),
    Skybox(SkyboxPass),
    Axis(AxisPass),
    BBox(BBoxPass),
    Linearize(LinearizePass),
    Outline(OutlinePass),
    PickObject(PickObjectPass),
}

impl RenderPass for RenderPassEnum {
    fn name(&self) -> &'static str {
        match self {
            RenderPassEnum::Mesh(p) => p.name(),
            RenderPassEnum::Light(p) => p.name(),
            RenderPassEnum::Skybox(p) => p.name(),
            RenderPassEnum::Axis(p) => p.name(),
            RenderPassEnum::BBox(p) => p.name(),
            RenderPassEnum::Linearize(p) => p.name(),
            RenderPassEnum::Outline(p) => p.name(),
            RenderPassEnum::PickObject(p) => p.name(),
        }
    }
    
    fn reads(&self) -> &[ResourceId] {
        match self {
            RenderPassEnum::Mesh(p) => p.reads(),
            RenderPassEnum::Light(p) => p.reads(),
            RenderPassEnum::Skybox(p) => p.reads(),
            RenderPassEnum::Axis(p) => p.reads(),
            RenderPassEnum::BBox(p) => p.reads(),
            RenderPassEnum::Linearize(p) => p.reads(),
            RenderPassEnum::Outline(p) => p.reads(),
            RenderPassEnum::PickObject(p) => p.reads(),
        }
    }

    fn writes(&self) -> &[ResourceId] {
        match self {
            RenderPassEnum::Mesh(p) => p.writes(),
            RenderPassEnum::Light(p) => p.writes(),
            RenderPassEnum::Skybox(p) => p.writes(),
            RenderPassEnum::Axis(p) => p.writes(),
            RenderPassEnum::BBox(p) => p.writes(),
            RenderPassEnum::Linearize(p) => p.writes(),
            RenderPassEnum::Outline(p) => p.writes(),
            RenderPassEnum::PickObject(p) => p.writes(),
        }
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
        match self {
            RenderPassEnum::Mesh(p) => {
                p.prepare(asset_mgr, world, globals, selected, input, ctx)
            }
            RenderPassEnum::Light(p) => {
                p.prepare(asset_mgr, world, globals, selected, input, ctx)
            }
            RenderPassEnum::Skybox(p) => {
                p.prepare(asset_mgr, world, globals, selected, input, ctx)
            }
            RenderPassEnum::Axis(p) => {
                p.prepare(asset_mgr, world, globals, selected, input, ctx)
            }
            RenderPassEnum::BBox(p) => {
                p.prepare(asset_mgr, world, globals, selected, input, ctx)
            }
            RenderPassEnum::Linearize(p) => {
                p.prepare(asset_mgr, world, globals, selected, input, ctx)
            }
            RenderPassEnum::Outline(p) => {
                p.prepare(asset_mgr, world, globals, selected, input, ctx)
            }
            RenderPassEnum::PickObject(p) => {
                p.prepare(asset_mgr, world, globals, selected, input, ctx)
            }
        }
    }

    fn execute(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        ctx: &mut RenderContext,
        asset_mgr: &AssetManager,
    ) {
        match self {
            RenderPassEnum::Mesh(p) => p.execute(encoder, ctx, asset_mgr),
            RenderPassEnum::Light(p) => p.execute(encoder, ctx, asset_mgr),
            RenderPassEnum::Skybox(p) => p.execute(encoder, ctx, asset_mgr),
            RenderPassEnum::Axis(p) => p.execute(encoder, ctx, asset_mgr),
            RenderPassEnum::BBox(p) => p.execute(encoder, ctx, asset_mgr),
            RenderPassEnum::Linearize(p) => p.execute(encoder, ctx, asset_mgr),
            RenderPassEnum::Outline(p) => p.execute(encoder, ctx, asset_mgr),
            RenderPassEnum::PickObject(p) => p.execute(encoder, ctx, asset_mgr),
        }
    }
}

