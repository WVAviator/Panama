mod pty_error;
mod pty_instance;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use self::{pty_error::PtyError, pty_instance::PtyInstance};
use portable_pty::{native_pty_system, PtySize, PtySystem};

pub struct ApplicationState {
    pty_system: Arc<Mutex<Box<dyn PtySystem + Send>>>,
    instances: Arc<Mutex<HashMap<u32, PtyInstance>>>,
}

impl ApplicationState {
    pub fn create() -> Result<Self, PtyError> {
        let mut pty_system = native_pty_system();

        let mut pair = pty_system
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| {
                PtyError::CreationError(format!("Unable to create pseudoterminal pair.\n{:?}", e))
            })?;

        let initial_instance = PtyInstance::create(pair)?;

        let instances = Arc::new(Mutex::new(HashMap::new()));
        {
            instances.lock().unwrap().insert(0, initial_instance);
        }

        Ok(ApplicationState {
            pty_system: Arc::new(Mutex::new(pty_system)),
            instances,
        })
    }

    pub fn create_instance(&self) -> Result<u32, PtyError> {
        let mut instances = self.instances.lock().unwrap();
        let mut pty_system = self.pty_system.lock().unwrap();

        let mut pair = pty_system
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| {
                PtyError::CreationError(format!("Unable to create pseudoterminal pair.\n{:?}", e))
            })?;

        let instance_id = instances.len() as u32;
        let instance = PtyInstance::create(pair)?;
        instances.insert(instance_id, instance);

        Ok(instance_id)
    }

    pub fn destroy_instance(&self, instance_id: u32) {
        let mut instances = self.instances.lock().unwrap();
        if let Some(instance) = instances.get_mut(&instance_id) {
            instance.destroy();
        }
    }
}
