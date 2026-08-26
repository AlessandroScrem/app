use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};

pub type EntityId = u64;
pub type QueryId = u64;

#[derive(Clone, Debug)]
pub enum Query { Hierarchy, Entity { entity: EntityId }, Children { parent: EntityId }, Inspector { entity: EntityId }, Selection, Settings, Statistics }
#[derive(Clone, Debug)]
pub struct QueryRequest { pub id: QueryId, pub query: Query }
#[derive(Clone, Debug)]
pub struct QueryResponse { pub id: QueryId, pub result: QueryResult }
#[derive(Clone, Debug)]
pub enum QueryResult { Hierarchy(HierarchyData), Entity(Option<EntityData>), Children(Vec<EntityData>), Inspector(Option<InspectorData>), Selection(Vec<EntityId>), Settings(EditorSettingsData), Statistics(EditorStatisticsData) }
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
#[derive(Clone, Debug)]
pub struct EditorSettingsData { pub light_enable: bool, pub ibl_enable: bool, pub skybox_enable: bool, pub skybox_enable_blur: bool, pub axis_enable: bool, pub bbox_enable: bool, pub bbox_axis_aligned: bool, pub mips_cp: bool, pub env_rotation: f32, pub debug_code: u32, pub exposure: f32, pub ibl_intensity: f32, pub tonemap_filter: u32, pub camera_fov: f32, pub camera_distance: f32, pub camera_near: f32, pub camera_far: f32, pub adapter_name: String, pub fps: f32, pub frametime: f32, pub root_nodes: usize, pub opaque_draw_calls: usize, pub opaque_instances: usize, pub transmission_draw_calls: usize, pub transmission_instances: usize }
#[derive(Clone, Debug)]
pub enum EditorCommand { Select { entities: Vec<EntityId> }, SetTransform { entity: EntityId, transform: TransformData }, SetName { entity: EntityId, name: String }, SetLight { entity: EntityId, light: LightData }, Delete { entities: Vec<EntityId> }, BeginTransformEdit { entity: EntityId }, EndTransformEdit { entity: EntityId }, AddLight, AddParent { entity: EntityId }, SetEntityEnabled { entity: EntityId, enabled: bool }, LoadGltf { path: std::path::PathBuf }, OpenScene { path: std::path::PathBuf }, SaveScene, SaveSceneAs { path: std::path::PathBuf }, ClearScene, Exit, SetLightEnable(bool), SetIblEnable(bool), SetSkyboxEnable(bool), SetSkyboxBlur(bool), SetAxisEnable(bool), SetBoundingBoxEnable(bool), SetBoundingBoxAxisAligned(bool), SetMipsWithCompute(bool), SetEnvironmentRotation(f32), SetDebugCode(u32), SetExposure(f32), SetIblIntensity(f32), SetTonemap(u32), RecenterCamera, SetCameraFov(f32), SetCameraDistance(f32), SetCameraNearFar { near: f32, far: f32 }, AddIbl { path: std::path::PathBuf } }
#[derive(Clone, Debug)]
pub enum EditorEvent { EntityCreated { entity: EntityId }, EntityDeleted { entity: EntityId }, TransformChanged { entity: EntityId, transform: TransformData }, NameChanged { entity: EntityId, name: String }, LightChanged { entity: EntityId }, SelectionChanged { entities: Vec<EntityId> }, SceneChanged, SettingsChanged, StatisticsChanged }
#[derive(Clone)]
pub struct EditorQueryClient { sender: Sender<QueryRequest>, next_id: std::sync::Arc<AtomicU64> }
impl EditorQueryClient { pub fn request(&self, query: Query) -> QueryId { let id = self.next_id.fetch_add(1, Ordering::Relaxed); self.sender.send(QueryRequest { id, query }).expect("editor service query channel disconnected"); id } }
#[derive(Clone)]
pub struct EditorCommandClient { sender: Sender<EditorCommand> }
impl EditorCommandClient { pub fn send(&self, command: EditorCommand) { self.sender.send(command).expect("editor service command channel disconnected"); } }
pub struct EditorEventReceiver { receiver: Receiver<EditorEvent> }
impl EditorEventReceiver { pub fn try_recv(&self) -> Option<EditorEvent> { self.receiver.try_recv().ok() } }
pub struct EditorConnection { pub queries: EditorQueryClient, pub commands: EditorCommandClient, pub events: EditorEventReceiver, responses: Receiver<QueryResponse> }
impl EditorConnection { pub fn new() -> (Self, EditorServiceChannels) { let (query_tx, query_rx) = mpsc::channel(); let (response_tx, response_rx) = mpsc::channel(); let (command_tx, command_rx) = mpsc::channel(); let (event_tx, event_rx) = mpsc::channel(); (Self { queries: EditorQueryClient { sender: query_tx, next_id: std::sync::Arc::new(AtomicU64::new(1)) }, commands: EditorCommandClient { sender: command_tx }, events: EditorEventReceiver { receiver: event_rx }, responses: response_rx }, EditorServiceChannels { query_rx, response_tx, command_rx, event_tx }) } pub fn try_recv_response(&self) -> Option<QueryResponse> { self.responses.try_recv().ok() } }
pub struct EditorServiceChannels { pub query_rx: Receiver<QueryRequest>, pub response_tx: Sender<QueryResponse>, pub command_rx: Receiver<EditorCommand>, pub event_tx: Sender<EditorEvent> }
