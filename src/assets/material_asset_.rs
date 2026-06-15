use super::*;
use crate::renderer::uniform::MaterialUniform;
use crate::uniform::Mat3Std140;
use std::cell::Cell;

#[derive(Debug, Default, Hash, Eq, PartialEq, Clone)]
pub enum ShaderId {
    #[default]
    Pbr,
}

#[derive(Clone)]
struct MaterialAsset {
    desc: MaterialDesc,
    ref_count: Cell<u32>,
}

impl HasStats for MaterialAssets {
    fn get_stats(&self) -> ResourceStats {
        self.stats.clone()
    }
}

#[derive(Default)]
pub struct MaterialAssets {
    storage: SlotMap<MaterialId, MaterialAsset>,
    stats: ResourceStats,
}

impl MaterialAssets {
    pub fn get_or_create(&mut self, desc: MaterialDesc) -> MaterialId {
        match self.find_duplicate(&desc) {
            Some(id) => {
                let mat = &self.storage[id];
                mat.ref_count.set(mat.ref_count.get() + 1);
                self.stats.add_shared();
                id
            }
            None => {
                let id = self.storage.insert(MaterialAsset {
                    desc: desc.clone(),
                    ref_count: Cell::new(1),
                });
                if self.storage[id].desc.name.is_empty() {
                    self.storage[id].desc.name = id.to_string();
                }
                self.stats.add(MaterialDesc::estimated_size());
                id
            }
        }
    }

    fn find_duplicate(&self, desc: &MaterialDesc) -> Option<MaterialId> {
        self.storage.iter().find_map(
            |(id, asset)| {
                if asset.desc == *desc { Some(id) } else { None }
            },
        )
    }

    pub fn get_desc(&self, id: MaterialId) -> Option<&MaterialDesc> {
        self.storage.get(id).map(|m| &m.desc)
    }

    pub fn contains_key(&self, id: MaterialId) -> bool {
        self.storage.contains_key(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (MaterialId, &MaterialDesc)> {
        self.storage.iter().map(|(id, asset)| (id, &asset.desc))
    }

    pub fn remove(&mut self, id: MaterialId, texture_asset: &mut TextureAssets) {
        if let Some(asset) = self.storage.get(id) {
            let count = asset.ref_count.get();

            if count > 1 {
                asset.ref_count.set(count - 1);
                self.stats.remove_sahred();
            } else {
                let removed = self.storage.remove(id).unwrap();
                let desc = removed.desc;
                // remove textures from slots
                // TODO: Remove from here
                for slot in MaterialTextureSlot::ALL {
                    if let Some(id) = desc.texture(slot) {
                        texture_asset.remove(id);
                        debug!("Remove texture slot {:?}", slot);
                    }
                }
                debug!("Remove material id {:?}", id);
                self.stats.remove(MaterialDesc::estimated_size());
            }
        }
    }

    pub fn update(&mut self, id: MaterialId, desc: &MaterialDesc) {
        if let Some(asset) = &mut self.storage.get_mut(id) {
            debug!("Update material id {:?} with desc {:?}", id, desc);
            asset.desc = desc.clone();
        } else {
            warn!("material id {} not found", id);
        }
    }
}

fn gen_transform_array(desc: &MaterialDesc) -> [Mat3Std140; MATERIAL_TEXTURE_COUNT] {
    std::array::from_fn(|i| {
        let slot = MaterialTextureSlot::ALL[i];

        desc.uvtransform(slot).unwrap_or_default().to_mat3_std140()
    })
}

impl From<&MaterialDesc> for MaterialUniform {
    fn from(value: &MaterialDesc) -> Self {
        let (alpha_mode, alpha_cutoff) = AlphaMode::to_uniform(value.alpha_mode);
        let is_trasmissive = value.is_transmissive().into();
        let transmission_factor = Transmission::to_uniform(value.transmission);

        let is_volume = value.is_volume().into();
        let (attenuation_distance, thickness_factor, attenuation_color) =
            Volume::to_uniform(value.volume);

        let texture_transforms = gen_transform_array(value);

        let is_sheen = value.is_sheen().into();
        let (sheen_color_factor, sheen_roughness_factor) = Sheen::to_uniform(value.sheen);

        Self {
            color_factor: value.base_color_factor.into(),
            emissive_factor: value.emissive_factor.into(),
            metallic_factor: value.metallic_factor,
            roughness_factor: value.roughness_factor,
            normal_scale: value.normal_scale,
            occlusion_strength: value.occlusion_strength,
            texture_flags: value.texture_set.texture_flags(),
            coord_flags: value.texture_set.coord_flags(),
            alpha_mode,
            alpha_cutoff,
            transmission_factor,
            is_trasmissive,
            is_volume,
            attenuation_distance,
            thickness_factor,
            attenuation_color,
            ior: value.ior,
            texture_transforms,
            is_sheen,
            sheen_color_factor,
            sheen_roughness_factor,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_material() {
        let mut materials = MaterialAssets::default();

        let desc = MaterialDesc::default();

        let id = materials.get_or_create(desc);

        assert!(materials.contains_key(id));
    }

    #[test]
    fn should_remove_material() {
        let mut materials = MaterialAssets::default();
        let mut texture_asset = TextureAssets::new();

        let desc = MaterialDesc::default();

        let _ = materials.get_or_create(desc.clone());
        let id = materials.get_or_create(desc);

        materials.remove(id, &mut texture_asset);
        assert!(materials.contains_key(id));
        assert!(materials.get_desc(id).is_some());

        materials.remove(id, &mut texture_asset);
        assert_eq!(materials.contains_key(id), false);
        assert!(materials.get_desc(id).is_none());
    }

    #[test]
    fn should_remove_textures_from_slot() {
        let mut materials = MaterialAssets::default();
        let mut texture_asset = TextureAssets::new();
        let path = Some(PathBuf::from("albedo.png"));

        let mut desc = MaterialDesc::default();
        desc.set_texture(
            &mut texture_asset,
            MaterialTextureSlot::BaseColor,
            path,
            0,
            None,
        );

        let id = materials.get_or_create(desc);
        let mat_desc = materials.get_desc(id).unwrap();

        let tex_id = mat_desc.texture(MaterialTextureSlot::BaseColor).unwrap();
        assert!(texture_asset.contains_key(tex_id));

        materials.remove(id, &mut texture_asset);
        assert_eq!(texture_asset.contains_key(tex_id), false);
    }

    #[test]
    fn should_have_stats() {
        fn assert_impl<T: HasStats>() {}
        assert_impl::<MaterialAssets>();
    }

    #[test]
    fn should_increment_stats_on_add() {
        let mut materials = MaterialAssets::default();
        let initial_stats = materials.get_stats();

        let desc = MaterialDesc::default();

        let _ = materials.get_or_create(desc);

        let new_stats = materials.get_stats();

        assert!(new_stats.count > initial_stats.count);
        assert!(new_stats.estimated_bytes > initial_stats.estimated_bytes);
    }

    #[test]
    fn should_decrements_stats_on_remove() {
        let mut materials = MaterialAssets::default();
        let mut textures = TextureAssets::new();

        let initial_stats = materials.get_stats();

        let desc = MaterialDesc::default();

        let id = materials.get_or_create(desc);

        materials.remove(id, &mut textures);
        let new_stats = materials.get_stats();

        assert_eq!(new_stats.count, initial_stats.count);
        assert_eq!(new_stats.estimated_bytes, initial_stats.estimated_bytes);
    }

    #[test]
    fn should_not_remove_shared_from_asset() {
        let mut materials = MaterialAssets::default();
        let mut textures = TextureAssets::new();

        let initial_stats = materials.get_stats();

        let desc = MaterialDesc::default();

        let _ = materials.get_or_create(desc.clone());
        let id = materials.get_or_create(desc);

        materials.remove(id, &mut textures);

        // now will remove ..
        materials.remove(id, &mut textures);
        let new_stats = materials.get_stats();

        assert_eq!(new_stats.count, initial_stats.count);
        assert_eq!(new_stats.estimated_bytes, initial_stats.estimated_bytes);
    }
}
