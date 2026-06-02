use std::collections::HashMap;

use super::GlobalAssetId;

#[derive(Default)]
pub struct DependencyGraph {
    // owner -> dependencies
    forward: HashMap<GlobalAssetId, Vec<GlobalAssetId>>,

    // dependency -> owners
    reverse: HashMap<GlobalAssetId, Vec<GlobalAssetId>>,
}

impl DependencyGraph {
    pub fn add(&mut self, owner: GlobalAssetId, dependency: GlobalAssetId) {
        self.forward.entry(owner).or_default().push(dependency);

        self.reverse.entry(dependency).or_default().push(owner);
    }

    pub fn dependencies_of(&self, owner: GlobalAssetId) -> Vec<GlobalAssetId> {
        self.forward.get(&owner).cloned().unwrap_or_default()
    }

    pub fn users_of(&self, dependency: GlobalAssetId) -> Vec<GlobalAssetId> {
        self.reverse.get(&dependency).cloned().unwrap_or_default()
    }

    pub fn remove_asset(&mut self, id: GlobalAssetId) {
        if let Some(deps) = self.forward.remove(&id) {
            for dep in deps {
                if let Some(users) = self.reverse.get_mut(&dep) {
                    users.retain(|v| *v != id);

                    if users.is_empty() {
                        self.reverse.remove(&dep);
                    }
                }
            }
        }

        if let Some(users) = self.reverse.remove(&id) {
            for owner in users {
                if let Some(deps) = self.forward.get_mut(&owner) {
                    deps.retain(|v| *v != id);

                    if deps.is_empty() {
                        self.forward.remove(&owner);
                    }
                }
            }
        }
    }
}
