use std::collections::{HashMap, HashSet};

use crate::assets::asset_id::GlobalAssetId;

#[derive(Default)]
pub struct DependencyGraph {
    dependencies: HashMap<GlobalAssetId, HashSet<GlobalAssetId>>,
    dependents: HashMap<GlobalAssetId, HashSet<GlobalAssetId>>,
}

impl DependencyGraph {
    pub fn add_dependency(&mut self, owner: GlobalAssetId, dependency: GlobalAssetId) {
        self.dependencies
            .entry(owner)
            .or_default()
            .insert(dependency);

        self.dependents.entry(dependency).or_default().insert(owner);
    }

    pub fn remove_dependency(&mut self, owner: GlobalAssetId, dependency: GlobalAssetId) {
        if let Some(deps) = self.dependencies.get_mut(&owner) {
            deps.remove(&dependency);
        }

        if let Some(users) = self.dependents.get_mut(&dependency) {
            users.remove(&owner);
        }
    }

    pub fn is_used(&self, asset: GlobalAssetId) -> bool {
        self.dependents.get(&asset).is_some_and(|s| !s.is_empty())
    }

    pub fn users_of(&self, asset: GlobalAssetId) -> impl Iterator<Item = GlobalAssetId> + '_ {
        self.dependents
            .get(&asset)
            .into_iter()
            .flat_map(|s| s.iter().copied())
    }

    pub fn remove_asset(&mut self, asset: GlobalAssetId) {
        if let Some(deps) = self.dependencies.remove(&asset) {
            for dep in deps {
                if let Some(users) = self.dependents.get_mut(&dep) {
                    users.remove(&asset);
                }
            }
        }

        if let Some(users) = self.dependents.remove(&asset) {
            for user in users {
                if let Some(deps) = self.dependencies.get_mut(&user) {
                    deps.remove(&asset);
                }
            }
        }
    }
}
