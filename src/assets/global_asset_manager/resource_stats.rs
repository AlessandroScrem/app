#[derive(Default, Debug, Clone)]
pub struct ResourceStats {
    pub count: usize,
    pub shared: usize,
    pub estimated_bytes: usize,
}