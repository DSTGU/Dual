#[derive(Clone)]
pub struct EngineConfig {
    pub hash: usize // In megabytes
}

impl Default for EngineConfig {
    fn default() -> Self {
        EngineConfig { hash: 256 }
    }
}

impl EngineConfig {
    // Use for tests that don't require massive data stores
    #[allow(dead_code)]
    pub fn thin() -> Self {
        EngineConfig { hash: 16 }
    }
}