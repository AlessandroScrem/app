use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct RecentFile {
    pub name: String,
    pub path: String,
}

impl From<PathBuf> for RecentFile {
    fn from(path: PathBuf) -> Self {
        let name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_owned();

        Self { name, path: path.to_string_lossy().into_owned() }
    }
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Settings {
    pub recent_files: Vec<RecentFile>,
}

impl Settings {
    const FILE: &'static str = crate::project_path!("settings.json");
    const MAX_RECENT: usize = 5;

    pub fn load() -> Self {
        match fs::read_to_string(Self::FILE) {
            Ok(data) => serde_json::from_str(&data).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self) -> std::io::Result<()> {
        let data = serde_json::to_string_pretty(self).expect("Failed to serialize settings");

        fs::write(Self::FILE, data)
    }

    pub fn add_recent_file(&mut self, file: RecentFile) {
        self.recent_files.retain(|f| f.path != file.path);

        self.recent_files.insert(0, file);


        self.recent_files.truncate(Self::MAX_RECENT);
    }
}
