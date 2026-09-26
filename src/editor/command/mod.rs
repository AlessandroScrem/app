mod asset;
mod camera;
mod entity;
mod global;
mod scene;
mod selection;

pub use asset::AssetCommand;
pub use camera::CameraCommand;
pub use entity::EntityCommand;
pub use global::GlobalCommand;
pub use scene::SceneCommand;
pub use selection::SelectionCommand;


#[derive(Clone, Debug)]
pub enum EditorCommand {
    Scene(SceneCommand),
    Selection(SelectionCommand),
    Entity(EntityCommand),
    Asset(AssetCommand),
    Camera(CameraCommand),
    Global(GlobalCommand),
    DragSelection((u32, u32), (u32, u32)),
    Exit,
}

impl From<EntityCommand> for EditorCommand {
    fn from(command: EntityCommand) -> Self {
        Self::Entity(command)
    }
}

impl From<AssetCommand> for EditorCommand {
    fn from(command: AssetCommand) -> Self {
        Self::Asset(command)
    }
}

impl From<CameraCommand> for EditorCommand {
    fn from(command: CameraCommand) -> Self {
        Self::Camera(command)
    }
}
impl From<GlobalCommand> for EditorCommand {
    fn from(command: GlobalCommand) -> Self {
        Self::Global(command)
    }
}

impl EditorCommand {
    pub fn settings_changed(&self) -> bool {
        match self {
            Self::Global(command) => command.settings_changed(),
            Self::Camera(command) => command.settings_changed(),
            Self::Asset(command) => command.settings_changed(),
            _ => false,
        }
    }
}