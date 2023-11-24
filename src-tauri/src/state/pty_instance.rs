use std::{
    io::{BufRead, BufReader},
    sync::{mpsc::Sender, Arc, Mutex},
    thread::JoinHandle,
};

use portable_pty::{PtyPair, PtySize};

use super::pty_error::PtyError;

pub struct PtyInstanceThread {
    pub instance_id: u32,
    pub pty_pair: Arc<Mutex<PtyPair>>,
    writer_thread: PtyWriterThread,
    reader_thread: PtyReaderThread,
}

impl PtyInstanceThread {
    pub fn new(
        size: PtySize,
        command: &str,
        handler: Box<dyn Fn(String) + Send>,
    ) -> Result<Self, PtyError> {
        let pty_system = portable_pty::native_pty_system();
        let pty_pair = pty_system.openpty(size).map_err(|e| {
            PtyError::CreationError(format!("Unable to create pseudoterminal pair.\n{:?}", e))
        })?;

        let cmd = portable_pty::CommandBuilder::new(command);
        pty_pair.slave.spawn_command(cmd).map_err(|e| {
            PtyError::CreationError(format!(
                "Unable to spawn command in new pseudoterminal pair.\n{:?}",
                e
            ))
        })?;

        let pty_pair = Arc::new(Mutex::new(pty_pair));

        let writer_thread = PtyWriterThread::new(Arc::clone(&pty_pair));
        let reader_thread = PtyReaderThread::new(Arc::clone(&pty_pair), handler);

        Ok(PtyInstanceThread {
            instance_id: 0,
            pty_pair,
            writer_thread,
            reader_thread,
        })
    }

    pub fn write(&mut self, input: String) -> Result<(), PtyError> {
        self.writer_thread.write(input)?;

        Ok(())
    }

    pub fn resize(&mut self, size: PtySize) -> Result<(), PtyError> {
        self.pty_pair
            .lock()
            .unwrap()
            .master
            .resize(size)
            .map_err(|e| {
                PtyError::InternalError(format!("Error occurred while resizing pty.\n{:?}", e))
            })?;

        Ok(())
    }

    pub fn close(&mut self) -> Result<(), PtyError> {
        self.writer_thread.close()?;
        self.reader_thread.close()?;

        Ok(())
    }
}

enum PtyThreadMessage {
    Write(String),
    Interrupt,
}

struct PtyWriterThread {
    write_tx: Sender<PtyThreadMessage>,
    join_handle: Option<JoinHandle<()>>,
}

impl PtyWriterThread {
    pub fn new(pty_pair: Arc<Mutex<PtyPair>>) -> Self {
        let (write_tx, write_rx) = std::sync::mpsc::channel::<PtyThreadMessage>();
        let mut writer = pty_pair.lock().unwrap().master.take_writer().unwrap();

        let join_handle = std::thread::spawn(move || loop {
            match write_rx.recv() {
                Ok(PtyThreadMessage::Interrupt) => {
                    println!("Interrupting write thread.");
                    break;
                }
                Ok(PtyThreadMessage::Write(data)) => {
                    writer
                        .write_all(data.as_bytes())
                        .expect("Error occurred while writing to pty.");
                }
                _ => {}
            }
        });

        PtyWriterThread {
            write_tx,
            join_handle: Some(join_handle),
        }
    }

    pub fn write(&self, input: String) -> Result<(), PtyError> {
        self.write_tx
            .send(PtyThreadMessage::Write(input))
            .map_err(|e| {
                PtyError::InternalError(format!(
                    "Error occurred while sending write message to write thread.\n{:?}",
                    e
                ))
            })?;

        Ok(())
    }

    pub fn close(&mut self) -> Result<(), PtyError> {
        self.write_tx
            .send(PtyThreadMessage::Interrupt)
            .expect("Error occurred while sending interrupt message to write thread.");

        self.join_handle.take().unwrap().join().map_err(|e| {
            PtyError::InternalError(format!("Error occurred while joining pty thread.\n{:?}", e))
        })?;

        Ok(())
    }
}

struct PtyReaderThread {
    read_tx: Sender<PtyThreadMessage>,
    join_handle: Option<JoinHandle<()>>,
}

impl PtyReaderThread {
    pub fn new(pty_pair: Arc<Mutex<PtyPair>>, handler: Box<dyn Fn(String) + Send>) -> Self {
        let (read_tx, read_rx) = std::sync::mpsc::channel::<PtyThreadMessage>();

        let reader = pty_pair.lock().unwrap().master.try_clone_reader().unwrap();
        let mut buf_reader = BufReader::new(reader);

        let join_handle = std::thread::spawn(move || loop {
            match read_rx.try_recv() {
                Ok(PtyThreadMessage::Interrupt) => {
                    println!("Interrupting read thread.");
                    break;
                }
                _ => {}
            }

            let data = buf_reader
                .fill_buf()
                .expect("Error occurred during buffer read.");
            let len = data.len();

            let data =
                String::from_utf8(data.to_vec()).expect("Unable to parse read buffer into string.");
            buf_reader.consume(len);

            handler(data);
        });

        PtyReaderThread {
            read_tx,
            join_handle: Some(join_handle),
        }
    }

    pub fn close(&mut self) -> Result<(), PtyError> {
        self.read_tx
            .send(PtyThreadMessage::Interrupt)
            .expect("Error occurred while sending interrupt message to read thread.");

        self.join_handle.take().unwrap().join().map_err(|e| {
            PtyError::InternalError(format!("Error occurred while joining pty thread.\n{:?}", e))
        })?;

        Ok(())
    }
}
