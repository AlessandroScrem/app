use crate::app::domain::events::DomainEvent::Selection;
use crate::app::domain::events::SelectionEvent::SelectIbl;
use crate::assets::asset_manager::{AssetEventKind, AssetManager};
use crate::assets::material_asset::MaterialAsset;
use crate::assets::mesh_asset::MeshAsset;
use crate::assets::texture_asset::{TextureAsset, TextureDesc};
use crate::assets::texture_upload::load_cpu_textures_par;
use crate::assets::{IblAsset, IblId, TextureId};
use crate::engine::engine::EventBus;
use crate::engine::RuntimeEvent;
use crate::gpu::texture::GpuTextureBuilder;
use crate::gpu::{
    BindgroupLayoutKind, GpuCache, GpuContext, GpuManager, GpuMaterial, GpuMesh, GpuTextureCache,
    IblManager, ShadowManager,
};
use crate::renderer::{ImguiRender, MaterialPreviewRenderer};

pub(crate) fn sync_gpu_assets(
    asset_mgr: &mut AssetManager,
    bus: &mut EventBus,
    gpu_context: &GpuContext,
    gpu_cache: &mut GpuCache,
    gpu_manager: &mut GpuManager,
    ibl_manager: &mut IblManager,
    imgui_render: &mut ImguiRender,
    hdr_vec: &mut Vec<(TextureId, IblId)>,
    shadow_manager: &ShadowManager,
    material_preview_renderer: &mut MaterialPreviewRenderer,
) {
    let texture_cache = &mut gpu_cache.textures;
    let material_cache = &mut gpu_cache.material;
    let mesh_cache = &mut gpu_cache.mesh;
    let grouped = asset_mgr.drain_grouped_events();
    grouped.process_type::<TextureAsset, _>(|kind, events| match kind {
        AssetEventKind::Created | AssetEventKind::Updated => {
            let jobs: Vec<(TextureId, TextureDesc)> = events
                .iter()
                .filter_map(|ev| {
                    asset_mgr
                        .get::<TextureAsset>(ev.id)
                        .map(|a| (ev.id, a.desc.clone()))
                })
                .collect();
            for (id, data) in load_cpu_textures_par(jobs) {
                let texture = GpuTextureBuilder::from_cpu(data).build(&gpu_context.as_ref());
                register_gpu_texture(
                    texture_cache,
                    imgui_render,
                    &gpu_context.device,
                    id,
                    texture,
                );

                // A texture replacement invalidates bind groups that captured
                // its previous view. Rebuild only materials that reference it.
                let dependent_materials = asset_mgr
                    .iter::<MaterialAsset>()
                    .filter(|(_, material)| material.desc.get_textures().contains(&id))
                    .map(|(material_id, _)| material_id)
                    .collect::<Vec<_>>();
                for material_id in dependent_materials {
                    if let Some(material) = asset_mgr.get::<MaterialAsset>(material_id) {
                        let layout =
                            gpu_manager.get_bindgroup_layout(BindgroupLayoutKind::Material);
                        material_cache.insert(
                            material_id,
                            GpuMaterial::new(
                                texture_cache,
                                &material.desc,
                                &gpu_context.device,
                                layout,
                            ),
                        );
                    }
                }

                // Rebuild any IBL environment derived from this HDR texture.
                let affected_ibl = hdr_vec
                    .iter()
                    .copied()
                    .filter(|(hdr_id, _)| *hdr_id == id)
                    .collect::<Vec<_>>();
                for (hdr_id, ibl_id) in affected_ibl {
                    if let Some(hdr) = texture_cache.get(hdr_id) {
                        let ibl = ibl_manager.create(hdr, &gpu_context.as_ref());
                        ibl_manager.insert(ibl_id, ibl);
                        gpu_manager.replace_pbrmap_skybox_bindgroup(
                            ibl_manager.get(&ibl_id),
                            &shadow_manager,
                            &gpu_context.device,
                        );
                        material_preview_renderer.invalidate_environment();
                        bus.send_runtime(RuntimeEvent::UpdateIblMaps(ibl_id));
                    }
                }
            }
        }
        AssetEventKind::Removed => events.iter().for_each(|ev| {
            remove_gpu_texture(texture_cache, &mut imgui_render, ev.id);
        }),
        _ => {}
    });
    grouped.process_type::<IblAsset, _>(|kind, events| {
        if let AssetEventKind::Created = kind {
            events
                .iter()
                .filter_map(|ev| asset_mgr.get::<IblAsset>(ev.id).map(|a| (ev.id, a)))
                .for_each(|(id, asset)| {
                    if let Some(hdr) = texture_cache.get(asset.hrd_id) {
                        ibl_manager.insert(id, ibl_manager.create(hdr, &gpu_context.as_ref()));
                        hdr_vec.push((asset.hrd_id, id));
                        bus.send_domain(Selection(SelectIbl(id)));
                        bus.send_runtime(RuntimeEvent::UpdateIblMaps(id));
                    }
                });
        }
    });
    grouped.process_type::<MaterialAsset, _>(|kind, events| match kind {
        AssetEventKind::Created | AssetEventKind::Updated => events
            .iter()
            .filter_map(|ev| asset_mgr.get::<MaterialAsset>(ev.id).map(|a| (ev.id, a)))
            .for_each(|(id, asset)| {
                let layout = gpu_manager.get_bindgroup_layout(BindgroupLayoutKind::Material);
                material_cache.insert(
                    id,
                    GpuMaterial::new(texture_cache, &asset.desc, &gpu_context.device, layout),
                );
            }),
        AssetEventKind::Removed => events.iter().for_each(|ev| material_cache.remove(ev.id)),
        _ => {}
    });
    grouped.process_type::<MeshAsset, _>(|kind, events| match kind {
        AssetEventKind::Created | AssetEventKind::Updated => events
            .iter()
            .filter_map(|ev| asset_mgr.get::<MeshAsset>(ev.id).map(|a| (ev.id, a)))
            .for_each(|(id, asset)| {
                mesh_cache.insert(
                    id,
                    GpuMesh::new(
                        &gpu_context.device,
                        &asset.desc.vertices,
                        &asset.desc.indices,
                    ),
                )
            }),
        AssetEventKind::Removed => events.iter().for_each(|ev| mesh_cache.remove(ev.id)),
        _ => {}
    });
}

fn register_gpu_texture(
    cache: &mut GpuTextureCache,
    imgui: &mut ImguiRender,
    device: &wgpu::Device,
    id: TextureId,
    texture: crate::gpu::texture::GpuTexture,
) {
    cache.insert(id, texture);
    let texture = cache
        .get(id)
        .expect("registered GPU texture must be available");
    imgui.registry.add(&mut imgui.renderer, device, id, texture);
}

fn remove_gpu_texture(cache: &mut GpuTextureCache, imgui: &mut ImguiRender, id: TextureId) {
    cache.remove(id);
    imgui.registry.remove(&mut imgui.renderer, id);
}
