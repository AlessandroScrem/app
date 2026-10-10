use crate::assets::asset_manager::{AssetManager, ResourceStats};
use crate::assets::{IblAsset, MaterialAsset, MeshAsset, TextureAsset};
use crate::editor::{EditorResourceStatsData, EditorStatisticsData, ResourceStatsData};
use crate::engine::editor::EditorService;
use crate::gpu::{GpuCache, HasGpuStats, IblManager, ShadowManager};
use crate::renderer::SceneRenderer;

pub(crate) struct UiStatistics {
    last_update: std::time::Instant,
    smoothed_dt: f32,
}

impl Default for UiStatistics {
    fn default() -> Self {
        Self {
            last_update: std::time::Instant::now(),
            smoothed_dt: 1.0 / 60.0,
        }
    }
}

impl UiStatistics {
    pub(crate) fn update(
        &mut self,
        editor_service: &mut EditorService,
        scene_renderer: &SceneRenderer,
        asset_mgr: &AssetManager,
        gpu_cache: &GpuCache,
        shadow_manager: &ShadowManager,
        ibl_manager: &IblManager,
    ) {
        let now = std::time::Instant::now();
        let dt = now
            .duration_since(self.last_update)
            .as_secs_f32()
            .clamp(1.0 / 1000.0, 0.25);
        self.last_update = now;
        self.smoothed_dt = self.smoothed_dt * 0.9 + dt * 0.1;

        let frame = scene_renderer.get_render_stats();
        editor_service.set_statistics(EditorStatisticsData {
            fps: 1.0 / self.smoothed_dt,
            frametime: self.smoothed_dt,
            opaque_draw_calls: frame.opaque.draw_calls,
            opaque_instances: frame.opaque.instances,
            transmission_draw_calls: frame.transmission.draw_calls,
            transmission_instances: frame.transmission.instances,
        });

        editor_service.set_resource_stats(EditorResourceStatsData {
            textures: asset_stats(asset_mgr.get_stats::<TextureAsset>()),
            materials: asset_stats(asset_mgr.get_stats::<MaterialAsset>()),
            meshes: asset_stats(asset_mgr.get_stats::<MeshAsset>()),
            ibl: asset_stats(asset_mgr.get_stats::<IblAsset>()),
            gpu_textures: gpu_stats(gpu_cache.textures.get_stats()),
            gpu_materials: gpu_stats(gpu_cache.material.get_stats()),
            gpu_meshes: gpu_stats(gpu_cache.mesh.get_stats()),
            gpu_shadows: gpu_stats(shadow_manager.get_stats()),
            gpu_ibl: gpu_stats(ibl_manager.get_stats()),
        });
    }
}

fn asset_stats(stats: ResourceStats) -> ResourceStatsData {
    ResourceStatsData {
        count: stats.count,
        estimated_bytes: stats.estimated_bytes,
    }
}

fn gpu_stats(stats: crate::gpu::GpuResourceStats) -> ResourceStatsData {
    ResourceStatsData {
        count: stats.count,
        estimated_bytes: stats.estimated_bytes,
    }
}
