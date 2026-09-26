use legion::{Entity, EntityStore, IntoQuery};

use crate::EntityRawU64;
use crate::app::domain::events::DomainEvent::Selection;
use crate::app::domain::events::SelectionEvent;
use crate::app::{
    App,
    domain::events::{AssetEvent, CameraEvent, DomainEvent, EntityEvent, GlobalEvent, SceneEvent},
};
use crate::ecs::components::{
    BoundingBoxComponent, Hidden, HierarchyComponent, LightComponent, MeshComponent, TagComponent,
    TransformComponent,
};
use crate::editor::{
    BoundingBoxData, EditValue, EditorCommand, EditorEvent, EditorSettingsData, EntityCommand,
    EntityData, EntityId, HierarchyData, HierarchyNode, InspectorData, LightData, MeshData, Query,
    QueryResult, SceneSettingsData, TransformData,
};
use crate::engine::{editor::EditorBackend, engine::EventBus};

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
            Query::SceneSettings => QueryResult::SceneSettings(self.scene_settings()),
        }
    }

    fn command(&mut self, command: EditorCommand, bus: &mut EventBus) -> Vec<EditorEvent> {
        match command {
            EditorCommand::Exit => {
                bus.send_runtime(crate::engine::RuntimeEvent::CloseRequested);
                Vec::new()
            }
            EditorCommand::DragSelection(pos, size) => {
                bus.send_runtime(crate::engine::RuntimeEvent::ReadbackSelection(pos, size));
                Vec::new()
            }
            command => {
                let settings_changed = command.settings_changed();
                let mut events = self.editor_command(command, bus);

                if settings_changed {
                    events.push(EditorEvent::SettingsChanged);
                }

                events
            }
        }
    }

    fn editor_command(
        &mut self,
        command: crate::editor::EditorCommand,
        bus: &mut EventBus,
    ) -> Vec<EditorEvent> {
        match command {
            EditorCommand::Selection(command) => self.selection_command(command, bus),
            EditorCommand::Entity(command) => self.entity_command(command, bus),
            EditorCommand::Scene(command) => self.scene_command(command, bus),
            EditorCommand::Asset(command) => self.asset_command(command, bus),
            EditorCommand::Camera(command) => self.camera_command(command, bus),
            EditorCommand::Global(command) => self.global_command(command, bus),
            EditorCommand::DragSelection(..) => vec![],
            EditorCommand::Exit => vec![],
        }
    }

    fn entity_command(
        &mut self,
        command: crate::editor::EntityCommand,
        bus: &mut EventBus,
    ) -> Vec<EditorEvent> {
        match command {
            EntityCommand::SetTransform { entity, transform } => {
                if let Ok(mut entry) = self
                    .current_scene
                    .world
                    .entry_mut(EntityRawU64::from_raw_u64(entity))
                {
                    if let Ok(component) = entry.get_component_mut::<TransformComponent>() {
                        *component = TransformComponent {
                            position: transform.translation,
                            rotation: transform.rotation,
                            scale: transform.scale,
                        };
                    }
                }
                vec![EditorEvent::TransformChanged { entity, transform }]
            }
            EntityCommand::Edit { entity, edit } => match edit {
                EditValue::Transform(transform) => {
                    if let Ok(mut entry) = self
                        .current_scene
                        .world
                        .entry_mut(EntityRawU64::from_raw_u64(entity))
                    {
                        if let Ok(component) = entry.get_component_mut::<TransformComponent>() {
                            *component = TransformComponent {
                                position: transform.translation,
                                rotation: transform.rotation,
                                scale: transform.scale,
                            };
                        }
                    }
                    vec![EditorEvent::TransformChanged { entity, transform }]
                }
                EditValue::Name(name) => {
                    if let Ok(mut entry) = self
                        .current_scene
                        .world
                        .entry_mut(EntityRawU64::from_raw_u64(entity))
                    {
                        if let Ok(tag) = entry.get_component_mut::<TagComponent>() {
                            tag.name = name.clone();
                        }
                    }
                    vec![EditorEvent::NameChanged { entity, name }]
                }
                EditValue::Light(light) => {
                    let raw_entity = EntityRawU64::from_raw_u64(entity);
                    if let Ok(mut entry) = self.current_scene.world.entry_mut(raw_entity) {
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
                    vec![EditorEvent::LightChanged { entity, light }]
                }
            },
            EntityCommand::Remove { entities } => {
                for entity in entities {
                    bus.send_domain(DomainEvent::Entity(EntityEvent::RemoveEntity(
                        EntityRawU64::from_raw_u64(entity),
                    )));
                }
                Vec::new()
            }
            EntityCommand::BeginTransformEdit { entity } => {
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
                Vec::new()
            }
            EntityCommand::EndTransformEdit { entity } => {
                let entity = EntityRawU64::from_raw_u64(entity);
                self.transform_edit.take().filter(|(id, _)| *id == entity);
                Vec::new()
            }
            EntityCommand::AddLight => {
                bus.send_domain(DomainEvent::Entity(EntityEvent::AddLight));
                Vec::new()
            }
            EntityCommand::AddParent { entity } => {
                bus.send_domain(DomainEvent::Entity(EntityEvent::AddParent(
                    EntityRawU64::from_raw_u64(entity),
                )));
                Vec::new()
            }
            EntityCommand::SetEnabled { entity, enabled } => {
                bus.send_domain(DomainEvent::Entity(EntityEvent::DisableEntity(
                    EntityRawU64::from_raw_u64(entity),
                    !enabled,
                )));
                Vec::new()
            }
        }
    }

    fn selection_command(
        &mut self,
        command: crate::editor::SelectionCommand,
        bus: &mut EventBus,
    ) -> Vec<EditorEvent> {
        use crate::editor::SelectionCommand;
        match command {
            SelectionCommand::Select { entities } => {
                bus.send_domain(Selection(SelectionEvent::Select(entities)));
            }
        }
        Vec::new()
    }

    fn scene_command(
        &mut self,
        command: crate::editor::SceneCommand,
        bus: &mut EventBus,
    ) -> Vec<EditorEvent> {
        use crate::editor::SceneCommand;
        match command {
            SceneCommand::Open(path) => bus.send_domain(DomainEvent::Scene(SceneEvent::Open(path))),
            SceneCommand::Save => bus.send_domain(DomainEvent::Scene(SceneEvent::Save)),
            SceneCommand::SaveAs(path) => {
                bus.send_domain(DomainEvent::Scene(SceneEvent::SaveAs(path)))
            }
            SceneCommand::Clear => bus.send_domain(DomainEvent::Scene(SceneEvent::ClearScene)),
        }
        Vec::new()
    }

    fn asset_command(
        &mut self,
        command: crate::editor::AssetCommand,
        bus: &mut EventBus,
    ) -> Vec<EditorEvent> {
        use crate::editor::AssetCommand;
        match command {
            AssetCommand::LoadGltf(path) => {
                bus.send_domain(DomainEvent::Assets(AssetEvent::LoadGltf(path)))
            }
            AssetCommand::AddIbl(path) => {
                bus.send_domain(DomainEvent::Assets(AssetEvent::AddIbl(path)))
            }
        }
        Vec::new()
    }

    fn camera_command(
        &mut self,
        command: crate::editor::CameraCommand,
        bus: &mut EventBus,
    ) -> Vec<EditorEvent> {
        use crate::editor::CameraCommand;
        match command {
            CameraCommand::Recenter => {
                bus.send_domain(DomainEvent::Camera(CameraEvent::RecenterCamera))
            }
            CameraCommand::SetFov(value) => {
                bus.send_domain(DomainEvent::Camera(CameraEvent::CameraFov(value)))
            }
            CameraCommand::SetDistance(value) => {
                bus.send_domain(DomainEvent::Camera(CameraEvent::CameraDistance(value)))
            }
            CameraCommand::SetNearFar { near, far } => bus.send_domain(DomainEvent::Camera(
                CameraEvent::CameraNearFar((near.max(0.1), far.max(near + 0.1))),
            )),
        }
        Vec::new()
    }

    fn global_command(
        &mut self,
        command: crate::editor::GlobalCommand,
        bus: &mut EventBus,
    ) -> Vec<EditorEvent> {
        use crate::editor::GlobalCommand;
        match command {
            GlobalCommand::SetLightEnable(value) => {
                bus.send_domain(DomainEvent::Global(GlobalEvent::LightEnable(value)))
            }
            GlobalCommand::SetIblEnable(value) => {
                bus.send_domain(DomainEvent::Global(GlobalEvent::IblEnable(value)))
            }
            GlobalCommand::SetSkyboxEnable(value) => {
                bus.send_domain(DomainEvent::Global(GlobalEvent::SkyboxEnable(value)))
            }
            GlobalCommand::SetSkyboxBlur(value) => {
                bus.send_domain(DomainEvent::Global(GlobalEvent::SkyboxEnableBlur(value)))
            }
            GlobalCommand::SetAxisEnable(value) => {
                bus.send_domain(DomainEvent::Global(GlobalEvent::AxisEnable(value)))
            }
            GlobalCommand::SetBoundingBoxEnable(value) => {
                bus.send_domain(DomainEvent::Global(GlobalEvent::BboxEnable(value)))
            }
            GlobalCommand::SetBoundingBoxAxisAligned(value) => {
                bus.send_domain(DomainEvent::Global(GlobalEvent::BboxAxisAligned(value)))
            }
            GlobalCommand::SetMipsWithCompute(value) => {
                bus.send_domain(DomainEvent::Global(GlobalEvent::MipsCsEnable(value)))
            }
            GlobalCommand::SetEnvironmentRotation(value) => {
                bus.send_domain(DomainEvent::Global(GlobalEvent::EnvRotation(value)))
            }
            GlobalCommand::SetDebugCode(value) => {
                bus.send_domain(DomainEvent::Global(GlobalEvent::DebugCode(value)))
            }
            GlobalCommand::SetExposure(value) => {
                bus.send_domain(DomainEvent::Global(GlobalEvent::Exposure(value)))
            }
            GlobalCommand::SetIblIntensity(value) => {
                bus.send_domain(DomainEvent::Global(GlobalEvent::IblIntensity(value)))
            }
            GlobalCommand::SetTonemap(value) => {
                bus.send_domain(DomainEvent::Global(GlobalEvent::TonemapFilter(value)))
            }
        }
        Vec::new()
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
        let transform = entry
            .get_component::<TransformComponent>()
            .map(|t| TransformData {
                translation: t.position,
                rotation: t.rotation,
                scale: t.scale,
            })
            .unwrap_or(TransformData {
                translation: [0.0; 3],
                rotation: [0.0; 3],
                scale: [1.0; 3],
            });
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
        }
    }

    fn scene_settings(&self) -> SceneSettingsData {
        let recent = self
            .settings
            .recent_files
            .iter()
            .map(|i| (i.name.clone(), i.path.clone()))
            .collect();
        SceneSettingsData { recent }
    }
}
