use crate::state::pty_error::PtyError;

#[derive(serde::Serialize)]
pub struct PtyResponseError {
    error_type: String,
    message: String,
}

impl From<PtyError> for PtyResponseError {
    fn from(error: PtyError) -> Self {
        PtyResponseError {
            error_type: error.error_type(),
            message: error.message(),
        }
    }
}
