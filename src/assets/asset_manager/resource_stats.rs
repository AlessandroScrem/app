#[derive(Default, Debug, Clone)]
pub struct ResourceStats {
    pub count: usize,
    pub shared: usize,
    pub estimated_bytes: usize,
}

impl ResourceStats {
    pub fn add(&mut self, size: usize) {
        self.estimated_bytes += size;
        self.count += 1;
    }
    pub fn remove(&mut self, size: usize) {
        if self.count > 0 {
            let result = self.estimated_bytes.checked_sub(size).unwrap_or(0);
            self.estimated_bytes = result;
            self.count -= 1;
        }
    }

    pub fn add_shared(&mut self) {
        self.shared += 1;
    }
    pub fn remove_sahred(&mut self) {
        if self.shared > 0 {
            self.shared -= 1;
        }
    }
}