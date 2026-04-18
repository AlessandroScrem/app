use super::*;

use crate::app::app_impl::RuntimeContext;
use crate::assets::asset_manager::AssetManager;
use crate::entities::EntityRawU64;
use crate::gpu::GpuContext;
use crate::uniform::{CameraUniform, GlobalUniform};

use legion::{Entity, World};
use wgpu::{Device, Queue};

use crate::picking::PickObject;
use crate::renderer::renderpass::*;

use crate::Globals;

pub struct RenderContext<'a> {
    pub device: &'a Device,
    pub gpu_cache: &'a GpuCache,

    pub gpu_mgr: &'a GpuManager,
    pub pip_mgr: &'a PipelineManager,
    pub pickobject: &'a PickObject,
    pub target: &'a wgpu::TextureView,
}

pub struct SceneRenderer {
    pickobject: PickObject,
    default_pass: Vec<RenderPassEnum>,
}

impl SceneRenderer {
    pub fn new(
        gpu_context: &GpuContext,
    ) -> Self {
        let timer = std::time::Instant::now();
        info!("Initializing renderer...");

        let device = &gpu_context.device;
        let pickobject = PickObject::new(&device);


        debug!("Renderer initialized in {} ms", timer.elapsed().as_millis());

        let default_pass = vec![
            RenderPassEnum::Mesh(MeshPass::opaque()),
            RenderPassEnum::Skybox(SkyboxPass::new()),
            RenderPassEnum::BuildMipmaps(BuildMipmapsPass::new()),
            RenderPassEnum::Transmission(MeshPass::transmission()),
            RenderPassEnum::Light(LightPass::new()),
            RenderPassEnum::Axis(AxisPass::new()),
            RenderPassEnum::BBox(BoundingboxPass::new()),
            RenderPassEnum::Linearize(LinearizePass::new()),
            RenderPassEnum::Outline(OutlinePass::new()),
            RenderPassEnum::PickObject(PickObjectPass::new()),
        ];

        Self {
            pickobject,
            default_pass,
        }
    }

    pub fn get_hovered(&mut self, gpu_context: &GpuContext) -> Option<Entity> {
        self.pickobject.poll_readback(&gpu_context.device)
    }

    pub fn render(
        &mut self,
        runtime: &mut RuntimeContext,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        size: (u32, u32),
        asset_mgr: &AssetManager,
        world: &World,
        camera: &Camera,
        globals: &Globals,
        selected: Option<Entity>,
    ) {
        let RuntimeContext {
            gpu_context,
            gpu_manager,
            pipeline_manager,
            gpu_cache,
            input,
        } = runtime;

        // sync GpuCache Ids with assets Ids (meshes materials textures)
        // update skybox
        gpu_manager.sync_ibl(gpu_cache, gpu_context, asset_mgr);
        gpu_cache.sync_caches(gpu_context, gpu_manager, asset_mgr);

        
        let frame = FrameBuilder::build(
            world,
            &gpu_context.device,
            asset_mgr,
            selected,
            &self.pickobject,
            input,
            globals,
        );
        
        // Update uniform buffer data to gpu
        self.update_render_globals_to_gpu(
            gpu_context,
            gpu_manager,
            camera,
            globals,
            selected,
            size,
        );
        Self::update_meshes_materials_to_gpu(asset_mgr, gpu_context, gpu_cache, &frame);
        Self::update_lights_to_gpu(gpu_context, gpu_manager, &frame);

        let mut ctx = RenderContext {
            device: &gpu_context.device,
            gpu_cache: &gpu_cache,

            gpu_mgr: &gpu_manager,
            pip_mgr: &pipeline_manager,
            pickobject: &self.pickobject,
            target: &target,
        };


        for pass in &mut self.default_pass {
            pass.execute(encoder, &mut ctx, &frame);
        }
    }

    // TODO! move out of here
    fn update_render_globals_to_gpu(
        &self,
        gpu_context: &GpuContext,
        gpu_manager: &GpuManager,
        camera: &Camera,
        globals: &Globals,
        selected: Option<Entity>,
        size: (u32, u32),
    ) {
        let entity_id = match selected {
            Some(id) => (&id).as_raw_u64(),
            None => 0,
        };

        gpu_manager.update_camera(
            &gpu_context.queue,
            &CameraUniform::from_camera_size(camera, size),
        );
        gpu_manager.update_globals(
            &gpu_context.queue,
            &GlobalUniform::from_global_id(globals, entity_id),
        );
    }

    // TODO! move out of here
    fn update_meshes_materials_to_gpu(
        asset_mgr: &AssetManager,
        gpu_context: &GpuContext,
        gpu_cache: &mut GpuCache,
        frame: &FrameData,
    ) {
        use cgmath::SquareMatrix;
        // -------- Mesh --------
        let queue = &gpu_context.queue;

        fn gpu_update(
            asset_mgr: &AssetManager,
            gpu_cache: &GpuCache,
            queue: &Queue,
            meshdraw: &MeshDraw,
        ) {
            assert!(
                meshdraw.transform.determinant() > 0.0,
                "matrix determinant is negative"
            );

            let mut model = uniform::ModelUniform::new(meshdraw.transform);
            model.entity_id = meshdraw.entity_id.as_raw_u64();
            gpu_cache.mesh.update(&meshdraw.mesh, queue, &model);
            if let Some(material_desc) = asset_mgr.materials.get_desc(meshdraw.material) {
                let updated_uniform = uniform::MaterialUniform::from(material_desc);
                gpu_cache
                    .material
                    .update(&meshdraw.material, queue, &updated_uniform);
            }
        }

        for meshdraw in frame.opaque.iter() {
            gpu_update(asset_mgr, gpu_cache, queue, meshdraw);
        }
        for meshdraw in frame.transmission.iter() {
            gpu_update(asset_mgr, gpu_cache, queue, meshdraw);
        }
    }

    // TODO! move out of here
    fn update_lights_to_gpu(gpu_context: &GpuContext, gpu_manager: &GpuManager, frame: &FrameData) {
        let queue = &gpu_context.queue;

        if let Some(light_uniform) = frame.lights {
            queue.write_buffer(
                gpu_manager.get_buffer(BufferKind::Light),
                0,
                bytemuck::bytes_of(&light_uniform),
            );
        }
    }
}
