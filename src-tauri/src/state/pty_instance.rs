use super::pty_error::PtyError;
use portable_pty::{Child, CommandBuilder, PtyPair};
use std::io::{BufRead, BufReader, Read, Write};

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

    pub fn read(&mut self) -> Result<String, PtyError> {
        let mut buf_reader = BufReader::new(&mut self.stdout);
        let data = buf_reader.fill_buf().map_err(|e| {
            PtyError::ReadError(format!(
                "Unable to read from pseudoterminal buffer.\n{:?}",
                e
            ))
        })?;

        if data.len() > 0 {
            let data_str = std::str::from_utf8(data).map_err(|e| {
                PtyError::ReadError(format!("Error converting buffer to UTF8.\n{:?}", e))
            })?;

            Ok(data_str.to_string())
        } else {
            Ok(String::new())
        }
    }

    pub fn write(&mut self, input: String) -> Result<String, PtyError> {
        self.stdin.write_all(input.as_bytes()).map_err(|e| {
            PtyError::WriteError(format!("Unable to write to pseudoterminal.\n{:?}", e))
        })?;

        return self.read();
    }
}
