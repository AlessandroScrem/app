use crate::assets::{MaterialId, material_desc::MaterialDesc};

#[derive(Clone, Debug)]
pub enum MaterialCommand {
    Update {
        id: MaterialId,
        desc: MaterialDesc,
    },
}

