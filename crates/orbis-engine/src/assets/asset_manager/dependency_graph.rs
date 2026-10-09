use std::collections::HashMap;

use crate::ResourceId;

#[derive(Default)]
pub struct DependencyGraph {
    // owner -> dependencies
    forward: HashMap<ResourceId, Vec<ResourceId>>,

    // dependency -> owners
    reverse: HashMap<ResourceId, Vec<ResourceId>>,
}

impl DependencyGraph {
    pub fn add(&mut self, owner: ResourceId, dependency: ResourceId) {
        self.forward.entry(owner).or_default().push(dependency);

        self.reverse.entry(dependency).or_default().push(owner);
    }

    pub fn dependencies_of(&self, owner: ResourceId) -> Vec<ResourceId> {
        self.forward.get(&owner).cloned().unwrap_or_default()
    }

    pub fn replace_dependencies(&mut self, owner: ResourceId, dependencies: &[ResourceId]) {
        if let Some(previous) = self.forward.remove(&owner) {
            for dependency in previous {
                if let Some(users) = self.reverse.get_mut(&dependency) {
                    users.retain(|user| *user != owner);
                    if users.is_empty() {
                        self.reverse.remove(&dependency);
                    }
                }
            }
        }

        if !dependencies.is_empty() {
            self.forward.insert(owner, dependencies.to_vec());
            for dependency in dependencies {
                self.reverse.entry(*dependency).or_default().push(owner);
            }
        }
    }

    /*
    pub fn users_of(&self, dependency: ResourceId) -> Vec<ResourceId> {
        self.reverse.get(&dependency).cloned().unwrap_or_default()
    }
    */

    pub fn remove_asset(&mut self, id: ResourceId) {
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
