#[derive(Debug)]
pub enum PtyError {
    CreationError(String),
    WriteError(String),
    ReadError(String),
    DestructionError(String),
}
