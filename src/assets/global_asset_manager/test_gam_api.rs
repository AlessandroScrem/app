use super::*;

/// -----------------------------
/// MOCK ASSETS (USER SIDE)
/// -----------------------------

#[derive(Clone)]
struct Texture {
    name: String,
}

#[derive(Clone)]
struct Material {
    name: String,
    albedo: Option<GlobalAssetId>,
    normal: Option<GlobalAssetId>,
}

#[derive(Clone)]
struct Mesh {
    name: String,
    material: Option<GlobalAssetId>,
}

/// -----------------------------
/// IMPLEMENT Asset TRAIT
/// -----------------------------

impl Asset for Texture {
    type Key = String;

    fn key(&self) -> &Self::Key {
        &self.name
    }
}

impl Asset for Material {
    type Key = String;

    fn key(&self) -> &Self::Key {
        &self.name
    }

    fn dependencies(&self) -> Vec<GlobalAssetId> {
        let mut deps = Vec::new();

        if let Some(a) = self.albedo {
            deps.push(a);
        }

        if let Some(n) = self.normal {
            deps.push(n);
        }

        deps
    }
}

impl Asset for Mesh {
    type Key = String;

    fn key(&self) -> &Self::Key {
        &self.name
    }

    fn dependencies(&self) -> Vec<GlobalAssetId> {
        let mut deps = Vec::new();

        if let Some(m) = self.material {
            deps.push(m);
        }

        deps
    }
}

#[test]
fn mesh_removal_reduces_material_refcount() {
    let mut mgr = GlobalAssetManager::new();

    let mat = mgr.add(Material {
        name: "Mat".into(),
        albedo: None,
        normal: None,
    });

    let mesh1 = mgr.add(Mesh {
        name: "M1".into(),
        material: Some(mat),
    });

    let mesh2 = mgr.add(Mesh {
        name: "M2".into(),
        material: Some(mat),
    });

    assert_eq!(mgr.ref_count.get(&mat), Some(&2));

    mgr.remove(mesh1);
    // ✔ mesh1 tolto → materiale ancora vivo
    assert_eq!(mgr.ref_count.get(&mat), Some(&1));

    mgr.remove(mesh2);
    // ✔ mesh2 tolto → materiale ancora vivo
    assert!(mgr.ref_count.get(&mat).is_none());
}

#[test]
fn material_removal_reduces_texture_refcount() {
    let mut mgr = GlobalAssetManager::new();

    let tex = mgr.add(Texture {
        name: "T.png".into(),
    });

    let mat1 = mgr.add(Material {
        name: "M1".into(),
        albedo: Some(tex),
        normal: None,
    });

    let mat2 = mgr.add(Material {
        name: "M2".into(),
        albedo: Some(tex),
        normal: None,
    });

    assert_eq!(mgr.ref_count.get(&tex), Some(&2));

    mgr.remove(mat1);
    // texture ancora viva
    assert_eq!(mgr.ref_count.get(&tex), Some(&1));

    mgr.remove(mat2);
    // texture distrutta
    assert!(mgr.ref_count.get(&tex).is_none());
}

#[test]
fn retain_release_behavior() {
    let mut mgr = GlobalAssetManager::new();

    let tex = mgr.add(Texture {
        name: "Tex.png".into(),
    });

    // add crea già l'asset vivo (ref = 1 o 0 dipende da design, ma NON 0 stabile)
    assert!(mgr.ref_count.get(&tex).is_some());

    // primo retain
    mgr.retain(tex);
    assert_eq!(mgr.ref_count.get(&tex), Some(&1));

    // release finale → DEVE essere rimosso
    mgr.release(tex);
    assert!(mgr.ref_count.get(&tex).is_none());
}

#[test]
fn dedup_chain_mesh_material_texture() {
    let mut mgr = GlobalAssetManager::new();

    let tex = mgr.add(Texture {
        name: "T.png".into(),
    });

    let mat = mgr.add(Material {
        name: "M".into(),
        albedo: Some(tex),
        normal: None,
    });

    let mesh1 = mgr.add(Mesh {
        name: "A".into(),
        material: Some(mat),
    });

    let mesh2 = mgr.add(Mesh {
        name: "A".into(),
        material: Some(mat),
    });

    // mesh dedup
    assert_eq!(mesh1, mesh2);
}
