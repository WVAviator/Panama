pub mod pty_error;
pub mod pty_instance;
pub mod pty_thread;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use self::{pty_error::PtyError, pty_thread::PtyThread};

pub struct ApplicationState {
    pub pty_thread_map: Arc<Mutex<HashMap<u32, PtyThread>>>,
}

impl ApplicationState {
    pub fn create() -> Result<Self, PtyError> {
        let pty_thread_map = Arc::new(Mutex::new(HashMap::new()));

        Ok(ApplicationState { pty_thread_map })
    }
}
