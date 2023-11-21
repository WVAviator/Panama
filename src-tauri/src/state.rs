pub mod pty_error;
pub mod pty_instance;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use self::pty_error::PtyError;

pub struct ApplicationState {
    pub pty_write_tx_map: Arc<Mutex<HashMap<u32, std::sync::mpsc::Sender<String>>>>,
}

impl ApplicationState {
    pub fn create() -> Result<Self, PtyError> {
        let pty_write_tx_map = Arc::new(Mutex::new(HashMap::new()));

        Ok(ApplicationState { pty_write_tx_map })
    }
}
