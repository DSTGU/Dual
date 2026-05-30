use std::path::Path;



pub struct EngineConfig {
    database_path: String,
    pub is_training: bool
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self { database_path: "./database.json".to_owned(), is_training: false }
    }
}

impl EngineConfig {
    pub fn get_path(&self) -> &Path {
        Path::new(&self.database_path)
    }

    pub fn set_path(&mut self, path: String) {
        self.database_path = path;
    }
}

