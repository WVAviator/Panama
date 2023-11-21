use super::pty_error::PtyError;
use portable_pty::{Child, CommandBuilder, PtyPair, PtySize};
use std::io::Write;

pub struct PtyInstance {
    pub pty_pair: PtyPair,
    child: Box<dyn Child + Send + Sync>,
    pub writer: Box<dyn Write + Send>,
}

impl PtyInstance {
    pub fn create(rows: u16, cols: u16) -> Result<Self, PtyError> {
        let pty_system = portable_pty::native_pty_system();
        let pty_pair = pty_system
            .openpty(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| {
                PtyError::CreationError(format!("Unable to create pseudoterminal pair.\n{:?}", e))
            })?;

        let cmd = CommandBuilder::new("zsh");
        let child = pty_pair.slave.spawn_command(cmd).map_err(|e| {
            PtyError::CreationError(format!(
                "Unable to spawn command in new pseudoterminal pair.\n{:?}",
                e
            ))
        })?;

        let writer = pty_pair.master.take_writer().map_err(|e| {
            PtyError::CreationError(format!(
                "Unable to take writer from psuedoterminal pair.\n{:?}",
                e
            ))
        })?;

        Ok(PtyInstance {
            pty_pair,
            child,
            writer,
        })
    }

    pub fn destroy(&mut self) -> Result<(), PtyError> {
        self.child.kill().map_err(|e| {
            PtyError::DestructionError(format!("Unable to kill child process.\n{:?}", e))
        })?;

        Ok(())
    }
}
