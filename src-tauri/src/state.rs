pub mod pty_error;
pub mod pty_instance;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use self::{pty_error::PtyError, pty_instance::PtyInstanceThread};

pub struct ApplicationState {
    pub pty_thread_map: Arc<Mutex<HashMap<u32, PtyInstanceThread>>>,
}

impl ApplicationState {
    pub fn create() -> Result<Self, PtyError> {
        let pty_thread_map = Arc::new(Mutex::new(HashMap::new()));

        Ok(ApplicationState { pty_thread_map })
    }
}
