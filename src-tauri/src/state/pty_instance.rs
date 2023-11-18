use super::pty_error::PtyError;
use portable_pty::{Child, CommandBuilder, PtyPair};
use std::io::{Read, Write};

pub struct PtyInstance {
    pty_pair: PtyPair,
    child: Box<dyn Child + Send + Sync>,
    stdout: Box<dyn Read + Send>,
    stdin: Box<dyn Write + Send>,
}

impl PtyInstance {
    pub fn create(pty_pair: PtyPair) -> Result<Self, PtyError> {
        let cmd = CommandBuilder::new("bash");
        let child = pty_pair.slave.spawn_command(cmd).map_err(|e| {
            PtyError::CreationError(format!(
                "Unable to spawn command in new pseudoterminal pair.\n{:?}",
                e
            ))
        })?;
        let stdout = pty_pair.master.try_clone_reader().map_err(|e| {
            PtyError::CreationError(format!(
                "Unable to clone reader from pseudoterminal pair.\n{:?}",
                e
            ))
        })?;
        let stdin = pty_pair.master.take_writer().map_err(|e| {
            PtyError::CreationError(format!(
                "Unable to take writer from psuedoterminal pair.\n{:?}",
                e
            ))
        })?;

        Ok(PtyInstance {
            pty_pair,
            child,
            stdout,
            stdin,
        })
    }

    pub fn destroy(&mut self) -> Result<(), PtyError> {
        self.child.kill().map_err(|e| {
            PtyError::DestructionError(format!("Unable to kill child process.\n{:?}", e))
        })?;

        Ok(())
    }
}
