#[derive(Debug, serde::Serialize)]
pub enum PtyError {
    CreationError(String),
    WriteError(String),
    ReadError(String),
    DestructionError(String),
}

impl PtyError {
    pub fn message(&self) -> String {
        match self {
            PtyError::CreationError(message) => message.to_string(),
            PtyError::WriteError(message) => message.to_string(),
            PtyError::ReadError(message) => message.to_string(),
            PtyError::DestructionError(message) => message.to_string(),
        }
    }

    pub fn error_type(&self) -> String {
        match self {
            PtyError::CreationError(_) => "CreationError".to_string(),
            PtyError::WriteError(_) => "WriteError".to_string(),
            PtyError::ReadError(_) => "ReadError".to_string(),
            PtyError::DestructionError(_) => "DestructionError".to_string(),
        }
    }
}
