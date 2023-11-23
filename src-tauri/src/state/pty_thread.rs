use std::{sync::mpsc::Sender, thread::JoinHandle};

use super::pty_error::PtyError;

pub struct PtyThread {
    pub pty_read_tx: Sender<PtyMessage>,
    pub pty_write_tx: Sender<PtyMessage>,
    pty_join_handle: Option<JoinHandle<()>>,
}

impl PtyThread {
    pub fn new(
        pty_read_tx: Sender<PtyMessage>,
        pty_write_tx: Sender<PtyMessage>,
        joinhandle: JoinHandle<()>,
    ) -> Self {
        PtyThread {
            pty_read_tx,
            pty_write_tx,
            pty_join_handle: Some(joinhandle),
        }
    }

    pub fn join(&mut self) -> Result<(), PtyError> {
        self.pty_join_handle.take().unwrap().join().map_err(|e| {
            PtyError::InternalError(format!("Error occurred while joining pty thread.\n{:?}", e))
        })?;

        Ok(())
    }
}

pub enum PtyMessage {
    Write(String),
    Resize(u16, u16),
    Interrupt,
}
