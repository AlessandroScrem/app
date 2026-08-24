#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct EntityRef(pub u64);

#[derive(Clone, Debug, Default)]
pub struct TransformState {
    pub translation: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
}

#[derive(Clone, Debug, Default)]
pub struct EntityState {
    pub entity: EntityRef,
    pub name: String,
    pub visible: bool,
    pub parent: Option<EntityRef>,
    pub children: Vec<EntityRef>,
    pub transform: Option<TransformState>,
}
