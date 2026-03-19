#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionConfig {
    pub entry_package: String,
    pub entry_function: String,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            entry_package: "main".to_string(),
            entry_function: "main".to_string(),
        }
    }
}
