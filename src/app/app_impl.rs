use super::{App, Application, HasAssetMgr};
use crate::EntityRawU64;
use crate::app::Settings;
use crate::app::application::AppRenderData;
use crate::app::domain::events::SelectionEvent::SelectIbl;
use crate::app::domain::events::{
    AssetEvent, CameraEvent, DomainEvent, EntityEvent, GlobalEvent, SceneEvent,
};
use crate::assets::TextureAsset;
use crate::assets::asset_manager::AssetManager;
use crate::assets::ibl_asset::IblAsset;
use crate::ecs::components::{
    BoundingBoxComponent, Hidden, HierarchyComponent, LightComponent, MeshComponent, TagComponent,
    TransformComponent,
};
use crate::editor::{
    BoundingBoxData, EditorCommand, EditorEvent, EditorSettingsData, EntityData, EntityId,
    HierarchyData, HierarchyNode, InspectorData, LightData, MeshData, Query, QueryResult,
    TransformData,
};
use crate::engine::editor::EditorBackend;
use crate::engine::engine::EventBus;
use crate::prelude::*;
use legion::Entity;

impl HasAssetMgr for App {
    fn asset_mgr_mut(&mut self) -> &mut AssetManager {
        &mut self.asset_mgr
    }
}
impl Application for App {
    fn init(&mut self, bus: &mut EventBus) {
        let timer = std::time::Instant::now();
        self.settings = Settings::load();
        crate::ecs::components::light::create(&mut self.current_scene.world);
        const HDRPATH: &str = crate::asset_path!("core/Cannon_Exterior.hdr");
        let hdr_texture_asset =
            TextureAsset::from_file(HDRPATH, crate::assets::texture_asset::TextureUsage::HDR16);
        let hdr_id = self.asset_mgr.add::<TextureAsset>(hdr_texture_asset);
        let ibl_id = self
            .asset_mgr
            .add::<IblAsset>(IblAsset::new(hdr_id, HDRPATH));
        bus.send_domain(DomainEvent::Selection(SelectIbl(ibl_id)));
        debug!("App initialized in {} ms", timer.elapsed().as_millis());
    }
    fn on_update(&mut self, bus: &mut EventBus) {
        self.update_domain_event(bus);
        self.current_scene.update_scene(bus, &self.globals);
    }
    fn on_resize(&mut self, width: u32, height: u32) {
        self.camera
            .set_aspect(width.max(1) as f32 / height.max(1) as f32);
    }
    fn on_drop(&mut self, path: std::path::PathBuf, bus: &mut EventBus) {
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("json") => bus.send_domain(DomainEvent::Scene(SceneEvent::Open(path))),
            Some("gltf") | Some("glb") => {
                bus.send_domain(DomainEvent::Assets(AssetEvent::LoadGltf(path)))
            }
            _ => {}
        }
    }
    fn render_data(&self) -> AppRenderData<'_> {
        AppRenderData {
            render_objects: &self.current_scene.render_objects,
            asset_mgr: &self.asset_mgr,
            camera: &self.camera,
            globals: &self.globals,
            selected: if self.selected.len() == 1 {
                self.selected.iter().next().copied()
            } else {
                None
            },
        }
    }
    fn on_close(&mut self) {
        let _ = self.settings.save();
        info!("Exit requested; App stopping");
    }
}

impl EditorBackend for App {
    fn query(&self, query: &Query) -> QueryResult {
        match query {
            Query::Hierarchy => QueryResult::Hierarchy(self.hierarchy_data()),
            Query::Entity { entity } => QueryResult::Entity(self.entity_data(*entity)),
            Query::Children { parent } => QueryResult::Children(self.children_data(*parent)),
            Query::Inspector { entity } => QueryResult::Inspector(self.inspector_data(*entity)),
            Query::Selection => QueryResult::Selection(self.editor_selection()),
            Query::Settings => QueryResult::Settings(self.editor_settings()),
            Query::Statistics => QueryResult::Statistics(Default::default()),
        }
    }
    fn command(&mut self, command: EditorCommand, bus: &mut EventBus) -> Vec<EditorEvent> {
        let settings_changed = matches!(
            &command,
            EditorCommand::SetLightEnable(_)
                | EditorCommand::SetIblEnable(_)
                | EditorCommand::SetSkyboxEnable(_)
                | EditorCommand::SetSkyboxBlur(_)
                | EditorCommand::SetAxisEnable(_)
                | EditorCommand::SetBoundingBoxEnable(_)
                | EditorCommand::SetBoundingBoxAxisAligned(_)
                | EditorCommand::SetMipsWithCompute(_)
                | EditorCommand::SetEnvironmentRotation(_)
                | EditorCommand::SetDebugCode(_)
                | EditorCommand::SetExposure(_)
                | EditorCommand::SetIblIntensity(_)
                | EditorCommand::SetTonemap(_)
                | EditorCommand::RecenterCamera
                | EditorCommand::SetCameraFov(_)
                | EditorCommand::SetCameraDistance(_)
                | EditorCommand::SetCameraNearFar { .. }
                | EditorCommand::AddIbl { .. }
        );
        let mut events = Vec::new();
        match command {
            EditorCommand::Select { entities } => {
                self.selected = entities
                    .into_iter()
                    .map(EntityRawU64::from_raw_u64)
                    .collect();
            }
            EditorCommand::SetTransform { entity, transform } => {
                if let Ok(mut e) = self
                    .current_scene
                    .world
                    .entry_mut(EntityRawU64::from_raw_u64(entity))
                {
                    if let Ok(t) = e.get_component_mut::<TransformComponent>() {
                        *t = TransformComponent {
                            position: transform.translation,
                            rotation: transform.rotation,
                            scale: transform.scale,
                        };
                    }
                }
                self.current_scene.update_scene(bus, &self.globals);
                events.push(EditorEvent::TransformChanged { entity, transform });
            }
            EditorCommand::SetName { entity, name } => {
                let entity_raw = EntityRawU64::from_raw_u64(entity);

                if let Ok(mut entry) = self.current_scene.world.entry_mut(entity_raw) {
                    if let Ok(tag) = entry.get_component_mut::<TagComponent>() {
                        tag.name = name.clone();
                    }
                }

                events.push(EditorEvent::NameChanged { entity, name });
            }
            EditorCommand::SetLight { entity, light } => {
                let e = EntityRawU64::from_raw_u64(entity);

                if let Ok(mut entry) = self.current_scene.world.entry_mut(e) {
                    if let Ok(component) = entry.get_component_mut::<LightComponent>() {
                        component.color = light.color;
                        component.directional = light.directional;
                        component.cast_shadow = light.cast_shadow;
                        component.entity_id = entity;
                        component.enabled = light.enabled;
                        component.frustum = light.frustum;
                        component.update_position(light.position);
                    }
                }

                self.current_scene.update_scene(bus, &self.globals);

                events.push(EditorEvent::LightChanged { entity, light });
            }
            EditorCommand::Delete { entities } => {
                for entity in entities {
                    bus.send_domain(DomainEvent::Entity(EntityEvent::RemoveEntity(
                        EntityRawU64::from_raw_u64(entity),
                    )));
                }
            }
            EditorCommand::BeginTransformEdit { entity } => {
                let entity = EntityRawU64::from_raw_u64(entity);
                if let Some(transform) = self.transform_for(entity) {
                    self.transform_edit = Some((
                        entity,
                        TransformData {
                            translation: transform.position,
                            rotation: transform.rotation,
                            scale: transform.scale,
                        },
                    ));
                }
            }
            EditorCommand::EndTransformEdit { entity } => {
                let entity = EntityRawU64::from_raw_u64(entity);
                self.transform_edit.take().filter(|(id, _)| *id == entity);
            }
            EditorCommand::AddLight => bus.send_domain(DomainEvent::Entity(EntityEvent::AddLight)),
            EditorCommand::AddParent { entity } => bus.send_domain(DomainEvent::Entity(
                EntityEvent::AddParent(EntityRawU64::from_raw_u64(entity)),
            )),
            EditorCommand::SetEntityEnabled { entity, enabled } => {
                bus.send_domain(DomainEvent::Entity(EntityEvent::DisableEntity(
                    EntityRawU64::from_raw_u64(entity),
                    !enabled,
                )))
            }
            EditorCommand::LoadGltf { path } => {
                bus.send_domain(DomainEvent::Assets(AssetEvent::LoadGltf(path)))
            }
            EditorCommand::OpenScene { path } => {
                bus.send_domain(DomainEvent::Scene(SceneEvent::Open(path)))
            }
            EditorCommand::SaveScene => bus.send_domain(DomainEvent::Scene(SceneEvent::Save)),
            EditorCommand::SaveSceneAs { path } => {
                bus.send_domain(DomainEvent::Scene(SceneEvent::SaveAs(path)))
            }
            EditorCommand::ClearScene => {
                bus.send_domain(DomainEvent::Scene(SceneEvent::ClearScene))
            }
            EditorCommand::Exit => bus.send_runtime(crate::engine::RuntimeEvent::CloseRequested),
            EditorCommand::SetLightEnable(v) => {
                bus.send_domain(DomainEvent::Global(GlobalEvent::LightEnable(v)))
            }
            EditorCommand::SetIblEnable(v) => {
                bus.send_domain(DomainEvent::Global(GlobalEvent::IblEnable(v)))
            }
            EditorCommand::SetSkyboxEnable(v) => {
                bus.send_domain(DomainEvent::Global(GlobalEvent::SkyboxEnable(v)))
            }
            EditorCommand::SetSkyboxBlur(v) => {
                bus.send_domain(DomainEvent::Global(GlobalEvent::SkyboxEnableBlur(v)))
            }
            EditorCommand::SetAxisEnable(v) => {
                bus.send_domain(DomainEvent::Global(GlobalEvent::AxisEnable(v)))
            }
            EditorCommand::SetBoundingBoxEnable(v) => {
                bus.send_domain(DomainEvent::Global(GlobalEvent::BboxEnable(v)))
            }
            EditorCommand::SetBoundingBoxAxisAligned(v) => {
                bus.send_domain(DomainEvent::Global(GlobalEvent::BboxAxisAligned(v)))
            }
            EditorCommand::SetMipsWithCompute(v) => {
                bus.send_domain(DomainEvent::Global(GlobalEvent::MipsCsEnable(v)))
            }
            EditorCommand::SetEnvironmentRotation(v) => {
                bus.send_domain(DomainEvent::Global(GlobalEvent::EnvRotation(v)))
            }
            EditorCommand::SetDebugCode(v) => {
                bus.send_domain(DomainEvent::Global(GlobalEvent::DebugCode(v)))
            }
            EditorCommand::SetExposure(v) => {
                bus.send_domain(DomainEvent::Global(GlobalEvent::Exposure(v)))
            }
            EditorCommand::SetIblIntensity(v) => {
                bus.send_domain(DomainEvent::Global(GlobalEvent::IblIntensity(v)))
            }
            EditorCommand::SetTonemap(v) => {
                bus.send_domain(DomainEvent::Global(GlobalEvent::TonemapFilter(v)))
            }
            EditorCommand::RecenterCamera => {
                bus.send_domain(DomainEvent::Camera(CameraEvent::RecenterCamera))
            }
            EditorCommand::SetCameraFov(v) => bus.send_domain(DomainEvent::Camera(
                CameraEvent::CameraFov(v),
            )),
            EditorCommand::SetCameraDistance(v) => {
                bus.send_domain(DomainEvent::Camera(CameraEvent::CameraDistance(v)))
            }
            EditorCommand::SetCameraNearFar { near, far } => bus.send_domain(DomainEvent::Camera(
                CameraEvent::CameraNearFar((near.max(0.1), far.max(near + 0.1))),
            )),
            EditorCommand::AddIbl { path } => {
                bus.send_domain(DomainEvent::Assets(AssetEvent::AddIbl(path)))
            }
        }
        if settings_changed {
            events.push(EditorEvent::SettingsChanged);
        }
        events
    }
    fn editor_scene_revision(&self) -> u64 {
        self.editor_scene_revision
    }
    fn editor_selection(&self) -> Vec<EntityId> {
        self.selected.iter().map(EntityRawU64::as_raw_u64).collect()
    }
    fn editor_entities(&self) -> Vec<EntityId> {
        let mut query = <Entity>::query();
        query
            .iter(&self.current_scene.world)
            .map(|e| e.as_raw_u64())
            .collect()
    }
}

impl App {
    fn transform_for(&self, entity: Entity) -> Option<TransformComponent> {
        self.current_scene
            .world
            .entry_ref(entity)
            .ok()
            .and_then(|e| e.get_component::<TransformComponent>().ok().cloned())
    }
    fn entity_data(&self, id: EntityId) -> Option<EntityData> {
        let entry = self
            .current_scene
            .world
            .entry_ref(EntityRawU64::from_raw_u64(id))
            .ok()?;
        let name = entry
            .get_component::<TagComponent>()
            .map(|tag| tag.name.clone())
            .unwrap_or_else(|_| "<unnamed>".into());
        Some(EntityData { id, name })
    }
    fn hierarchy_data(&self) -> HierarchyData {
        let mut nodes = Vec::new();
        let mut query = <(Entity, &HierarchyComponent)>::query();
        for (entity, hierarchy) in query.iter(&self.current_scene.world) {
            let entry = self.current_scene.world.entry_ref(*entity).ok();
            let name = self
                .entity_data(entity.as_raw_u64())
                .map(|d| d.name)
                .unwrap_or_else(|| "<unnamed>".into());
            let visible = entry
                .as_ref()
                .map(|e| e.get_component::<Hidden>().is_err())
                .unwrap_or(true);
            let is_light = entry
                .as_ref()
                .map(|e| e.get_component::<LightComponent>().is_ok())
                .unwrap_or(false);
            nodes.push(HierarchyNode {
                entity: entity.as_raw_u64(),
                parent: hierarchy.parent.map(|p| p.as_raw_u64()),
                name,
                visible,
                is_light,
            });
        }
        let mut query = <(Entity, &LightComponent)>::query();
        for (entity, light) in query.iter(&self.current_scene.world) {
            let name = self
                .entity_data(entity.as_raw_u64())
                .map(|d| d.name)
                .unwrap_or_else(|| "<unnamed>".into());
            nodes.push(HierarchyNode {
                entity: entity.as_raw_u64(),
                parent: None,
                name,
                visible: light.enabled,
                is_light: true,
            });
        }
        nodes.sort_by(|a, b| a.name.cmp(&b.name));
        HierarchyData { nodes }
    }
    fn children_data(&self, parent: EntityId) -> Vec<EntityData> {
        let Some(entry) = self
            .current_scene
            .world
            .entry_ref(EntityRawU64::from_raw_u64(parent))
            .ok()
        else {
            return Vec::new();
        };
        let Ok(hierarchy) = entry.get_component::<HierarchyComponent>() else {
            return Vec::new();
        };
        hierarchy
            .children
            .iter()
            .filter_map(|e| self.entity_data(e.as_raw_u64()))
            .collect()
    }
    fn inspector_data(&self, id: EntityId) -> Option<InspectorData> {
        let entry = self
            .current_scene
            .world
            .entry_ref(EntityRawU64::from_raw_u64(id))
            .ok()?;
        let name = entry
            .get_component::<TagComponent>()
            .map(|tag| tag.name.clone())
            .unwrap_or_else(|_| "<unnamed>".into());
        let transform = {
            if let Some(t) = entry.get_component::<TransformComponent>().ok() {
                TransformData {
                    translation: t.position,
                    rotation: t.rotation,
                    scale: t.scale,
                }
            } else {
                TransformData {
                    translation: [0.0, 0.0, 0.0],
                    rotation: [0.0, 0.0, 0.0],
                    scale: [1.0, 1.0, 1.0],
                }
            }
        };
        let mesh = entry
            .get_component::<MeshComponent>()
            .ok()
            .map(|mesh| MeshData {
                id: format!("{:?}", mesh.handle),
            });
        let bounding_box = entry
            .get_component::<BoundingBoxComponent>()
            .ok()
            .map(|bbox| BoundingBoxData {
                min: bbox.bounding_box.min,
                max: bbox.bounding_box.max,
                global_min: bbox.global_bounding_box.min,
                global_max: bbox.global_bounding_box.max,
            });
        let light = entry
            .get_component::<LightComponent>()
            .ok()
            .map(|light| LightData {
                position: light.get_position(),
                color: light.color,
                enabled: light.enabled,
                directional: light.directional,
                cast_shadow: light.cast_shadow,
                frustum: light.frustum,
            });
        Some(InspectorData {
            entity: id,
            name,
            transform,
            mesh,
            bounding_box,
            light,
        })
    }
    fn editor_settings(&self) -> EditorSettingsData {
        let (near, far) = self.camera.get_near_far();
        EditorSettingsData {
            light_enable: self.globals.light_enable,
            ibl_enable: self.globals.ibl_enable,
            skybox_enable: self.globals.skybox_enable,
            skybox_enable_blur: self.globals.skybox_enable_blur,
            axis_enable: self.globals.axis_enable,
            bbox_enable: self.globals.bbox_enable,
            bbox_axis_aligned: self.globals.bbox_axis_aligned,
            mips_cp: self.globals.mips_cp,
            env_rotation: self.globals.env_rotation,
            debug_code: self.globals.debug_code,
            exposure: self.globals.exposure,
            ibl_intensity: self.globals.ibl_intensity,
            tonemap_filter: self.globals.tonemap_filter,
            camera_fov: self.camera.get_fov(),
            camera_distance: self.camera.get_distance(),
            camera_near: near,
            camera_far: far,
            adapter_name: String::new(),
            fps: 0.0,
            frametime: 0.0,
            root_nodes: self
                .hierarchy_data()
                .nodes
                .iter()
                .filter(|n| n.parent.is_none())
                .count(),
            opaque_draw_calls: 0,
            opaque_instances: 0,
            transmission_draw_calls: 0,
            transmission_instances: 0,
        }
    }
}
