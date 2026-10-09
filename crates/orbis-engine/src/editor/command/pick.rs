#[derive(Clone, Debug)]
pub enum PickCommand {
    Region {
        origin: (u32, u32),
        size: (u32, u32),
    },
}
