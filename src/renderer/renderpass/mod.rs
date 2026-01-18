pub mod axis;
pub mod bbox;
pub mod light;
pub mod linearize;
pub mod mesh;
pub mod outline;
pub mod pickobject;
pub mod skybox;
pub mod imguipass;

pub use axis::AxisPass;
pub use bbox::BBoxPass;
pub use light::LightPass;
pub use linearize::LinearizePass;
pub use mesh::MeshPass;
pub use outline::OutlinePass;
pub use pickobject::PickObjectPass;
pub use skybox::SkyboxPass;
pub use imguipass::ImguiPass;

use crate::entities::EntityRawU64;
use crate::input::Input;
use crate::renderer::pipeline_manager::PipelineKind;
use crate::uniform::{LightUniform, MaterialUniform};
use crate::{BoundingBoxComponent, LightComponent};
use crate::{
    Camera, GlobalModelComponent, Globals, MeshComponent, renderer::gpu_renderer::RenderContext,
    uniform::ModelUniform,
};

pub use legion::query::IntoQuery;
pub use legion::{Entity, Resources, World};
use wgpu::IndexFormat;

pub enum RenderPassEnum {
    Mesh(MeshPass),
    Light(LightPass),
    Skybox(SkyboxPass),
    Axis(AxisPass),
    BBox(BBoxPass),
    Linearize(LinearizePass),
    Outline(OutlinePass),
    PickObject(PickObjectPass),
    Imgui(ImguiPass),
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
            RenderPassEnum::Imgui(p) => p.name(),
        }
    }

    fn prepare(
        &mut self,
        world: &World,
        res: &Resources,
        camera: &Camera,
        globals: &Globals,
        selected: Option<Entity>,
        input: &Input,
        ctx: &mut RenderContext,
    ) {
        match self {
            RenderPassEnum::Mesh(p) => p.prepare(world, res, camera, globals, selected, input, ctx),
            RenderPassEnum::Light(p) => {
                p.prepare(world, res, camera, globals, selected, input, ctx)
            }
            RenderPassEnum::Skybox(p) => {
                p.prepare(world, res, camera, globals, selected, input, ctx)
            }
            RenderPassEnum::Axis(p) => p.prepare(world, res, camera, globals, selected, input, ctx),
            RenderPassEnum::BBox(p) => p.prepare(world, res, camera, globals, selected, input, ctx),
            RenderPassEnum::Linearize(p) => {
                p.prepare(world, res, camera, globals, selected, input, ctx)
            }
            RenderPassEnum::Outline(p) => {
                p.prepare(world, res, camera, globals, selected, input, ctx)
            }
            RenderPassEnum::PickObject(p) => {
                p.prepare(world, res, camera, globals, selected, input, ctx)
            }
            RenderPassEnum::Imgui(p) => {
                p.prepare(world, res, camera, globals, selected, input, ctx)
            }
        }
    }

    fn execute(&mut self, encoder: &mut wgpu::CommandEncoder, ctx: &mut RenderContext) {
        match self {
            RenderPassEnum::Mesh(p) => p.execute(encoder, ctx),
            RenderPassEnum::Light(p) => p.execute(encoder, ctx),
            RenderPassEnum::Skybox(p) => p.execute(encoder, ctx),
            RenderPassEnum::Axis(p) => p.execute(encoder, ctx),
            RenderPassEnum::BBox(p) => p.execute(encoder, ctx),
            RenderPassEnum::Linearize(p) => p.execute(encoder, ctx),
            RenderPassEnum::Outline(p) => p.execute(encoder, ctx),
            RenderPassEnum::PickObject(p) => p.execute(encoder, ctx),
            RenderPassEnum::Imgui(p) => p.execute(encoder, ctx),
        }
    }
}

pub trait RenderPass {
    fn name(&self) -> &'static str;

    fn prepare(
        &mut self,
        world: &World,
        resources: &Resources,
        camera: &Camera,
        globals: &Globals,
        selected: Option<Entity>,
        input: &Input,
        ctx: &mut RenderContext,
    );

    fn execute(&mut self, encoder: &mut wgpu::CommandEncoder, ctx: &mut RenderContext);
}

