#[derive(Debug)]
pub enum PtyError {
    CreationError(String),
    DestructionError(String),
}
