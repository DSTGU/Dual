#[derive(Clone)]
pub struct EngineConfig {
    pub hash: usize, // In megabytes
    /// Soft node limit from the `SoftNodes` UCI option.
    /// `None` means no soft node limit (option set to 0).
    pub soft_nodes: Option<u64>,
}

impl Default for EngineConfig {
    fn default() -> Self {
        EngineConfig {
            hash: 256,
            soft_nodes: None,
        }
    }
}

impl EngineConfig {
    // Use for tests that don't require massive data stores
    #[allow(dead_code)]
    pub fn thin() -> Self {
        EngineConfig {
            hash: 16,
            soft_nodes: None,
        }
    }
}