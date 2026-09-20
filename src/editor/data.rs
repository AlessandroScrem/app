pub type EntityId = u64;

#[derive(Clone, Debug)]
pub struct EditorEdit<K, T> {
    pub key: K,
    pub value: T,
}

impl<K, T> EditorEdit<K, T> {
    pub fn new(key: K, value: T) -> Self {
        Self { key, value }
    }
}

#[derive(Clone, Debug)]
pub enum EditValue {
    Transform(TransformData),
    Light(LightData),
    Name(String),
}

#[derive(Clone, Debug)]
pub struct EntityData { pub id: EntityId, pub name: String }
#[derive(Clone, Debug, Default)]
pub struct HierarchyData { pub nodes: Vec<HierarchyNode> }
#[derive(Clone, Debug)]
pub struct HierarchyNode { pub entity: EntityId, pub parent: Option<EntityId>, pub name: String, pub visible: bool, pub is_light: bool }
#[derive(Clone, Debug)]
pub struct InspectorData { pub entity: EntityId, pub name: String, pub transform: TransformData, pub mesh: Option<MeshData>, pub bounding_box: Option<BoundingBoxData>, pub light: Option<LightData> }
#[derive(Clone, Debug)]
pub struct MeshData { pub id: String }
#[derive(Clone, Debug)]
pub struct BoundingBoxData { pub min: [f32; 3], pub max: [f32; 3], pub global_min: [f32; 3], pub global_max: [f32; 3] }
#[derive(Clone, Debug)]
pub struct LightData { pub position: [f32; 3], pub color: [f32; 3], pub enabled: bool, pub directional: bool, pub cast_shadow: bool, pub frustum: bool }
#[derive(Clone, Debug, PartialEq)]
pub struct TransformData { pub translation: [f32; 3], pub rotation: [f32; 3], pub scale: [f32; 3] }
#[derive(Clone, Debug, Default, PartialEq)]
pub struct EditorStatisticsData { pub fps: f32, pub frametime: f32, pub adapter_name: String, pub root_nodes: usize, pub opaque_draw_calls: u32, pub opaque_instances: u32, pub transmission_draw_calls: u32, pub transmission_instances: u32 }
#[derive(Clone, Debug, Default)]
pub struct SceneSettingsData { pub recent: Vec<(String, String)> }
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct EditorSettingsData { pub light_enable: bool, pub ibl_enable: bool, pub skybox_enable: bool, pub skybox_enable_blur: bool, pub axis_enable: bool, pub bbox_enable: bool, pub bbox_axis_aligned: bool, pub mips_cp: bool, pub env_rotation: f32, pub debug_code: u32, pub exposure: f32, pub ibl_intensity: f32, pub tonemap_filter: u32, pub camera_fov: f32, pub camera_distance: f32, pub camera_near: f32, pub camera_far: f32, pub adapter_name: String, pub fps: f32, pub frametime: f32, pub root_nodes: usize, pub opaque_draw_calls: usize, pub opaque_instances: usize, pub transmission_draw_calls: usize, pub transmission_instances: usize }
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum EditorEvent { EntityCreated { entity: EntityId }, EntityDeleted { entity: EntityId }, TransformChanged { entity: EntityId, transform: TransformData }, NameChanged { entity: EntityId, name: String }, LightChanged { entity: EntityId, light: LightData }, SelectionChanged { entities: Vec<EntityId> }, SceneChanged, SettingsChanged, StatisticsChanged }
