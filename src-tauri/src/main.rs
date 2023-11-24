// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod pty_response_error;
mod state;

use std::{
    io::{BufRead, BufReader},
    sync::Arc,
};

use portable_pty::PtySize;
use pty_response_error::PtyResponseError;
use state::{
    pty_error::PtyError,
    pty_instance::PtyInstance,
    pty_thread::{PtyMessage, PtyThread},
    ApplicationState,
};
use tauri::{command, LogicalSize, Manager, PhysicalSize, Size, State, Window};

#[derive(serde::Serialize)]
struct CreateResponse {
    instance_id: u32,
}

#[derive(serde::Serialize, Clone)]
struct ReadResponse {
    output: String,
}

#[tauri::command]
fn create(
    rows: u16,
    cols: u16,
    instance_id: u32,
    command: String,
    window: Window,
    state: State<'_, ApplicationState>,
) -> Result<CreateResponse, PtyResponseError> {
    let thread_map = Arc::clone(&state.pty_thread_map);

    if let Some(pty_thread) = thread_map
        .lock()
        .map_err(|e| {
            PtyError::InternalError(format!(
                "Error occurred while obtaining lock to threads map.\n{:?}",
                e
            ))
        })?
        .get(&instance_id)
    {
        println!("Instance with id {} already exists.", instance_id);
        pty_thread
            .pty_read_tx
            .send(PtyMessage::Resize(rows, cols))
            .map_err(|e| {
                PtyError::InternalError(format!(
                    "Error occurred while sending resize command to existing instance.\n{:?}",
                    e
                ))
            })?;

        pty_thread
            .pty_write_tx
            .send(PtyMessage::Write("\n".to_string()))
            .map_err(|e| {
                PtyError::InternalError(format!(
                    "Error occurred while sending command to existing instance.\n{:?}",
                    e
                ))
            })?;
        return Ok(CreateResponse { instance_id });
    }

    println!("Creating pty instance with id: {}", instance_id);

    let (write_tx, write_rx) = std::sync::mpsc::channel::<PtyMessage>();
    let (read_tx, read_rx) = std::sync::mpsc::channel::<PtyMessage>();

    let mut threads = thread_map.lock().map_err(|e| {
        PtyError::InternalError(format!(
            "Error occurred while obtaining lock to threads map.\n{:?}",
            e
        ))
    })?;

    let pty_thread = std::thread::spawn(move || {
        let instance = PtyInstance::create(rows, cols, &command).unwrap();
        let mut writer = instance.writer;

        let write_thread = std::thread::spawn(move || loop {
            match write_rx.recv() {
                Ok(PtyMessage::Write(input)) => {
                    writer
                        .write_all(input.as_bytes())
                        .expect("Unable to write to instance");
                }
                Ok(PtyMessage::Interrupt) => {
                    println!(
                        "Interrupting write thread in instance with id: {}",
                        instance_id
                    );
                    // Writing an EOF will unblock the read thread so that it can receive its interrupt message.
                    writer
                        .write_all(&[4])
                        .expect("Unable to write EOF to instance");
                    break;
                }
                _ => {
                    println!("Error occurred while receiving write message.");
                    break;
                }
            }
        });

        let reader = instance
            .pty_pair
            .master
            .try_clone_reader()
            .expect("Error occurred cloning reader from pty master.");

        let mut buf_reader = BufReader::new(reader);

        loop {
            match read_rx.try_recv() {
                Ok(PtyMessage::Interrupt) => {
                    println!(
                        "Interrupting read thread in instance with id: {}",
                        instance_id
                    );
                    break;
                }
                Ok(PtyMessage::Resize(rows, cols)) => {
                    instance
                        .pty_pair
                        .master
                        .resize(PtySize {
                            rows,
                            cols,
                            pixel_width: 0,
                            pixel_height: 0,
                        })
                        .expect("Error occurred resizing pty instance.");
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

            window
                .emit(
                    format!("read:{}", instance_id).as_str(),
                    ReadResponse { output: data },
                )
                .expect("Error occurred while emitting window read event.");
        }

        write_thread.join().expect("Unable to join write thread.");
    });

    threads.insert(instance_id, PtyThread::new(read_tx, write_tx, pty_thread));

    Ok(CreateResponse { instance_id })
}

#[tauri::command]
fn write(
    instance_id: u32,
    input: String,
    state: State<'_, ApplicationState>,
) -> Result<(), PtyResponseError> {
    let thread_map = Arc::clone(&state.pty_thread_map);
    let mut threads = thread_map.lock().map_err(|e| {
        PtyError::InternalError(format!(
            "Error occurred obtaining lock to instaces map.\n{:?}",
            e
        ))
    })?;
    let pty_thread = threads
        .get_mut(&instance_id)
        .ok_or(PtyError::WriteError(format!(
            "Instance with id {} not found.",
            &instance_id
        )))?;
    let write_tx = &pty_thread.pty_write_tx;
    write_tx.send(PtyMessage::Write(input)).map_err(|e| {
        PtyError::InternalError(format!(
            "Error occurred transmitting write to instance thread.\n{:?}",
            e
        ))
    })?;

    Ok(())
}

#[tauri::command]
fn destroy(instance_id: u32, state: State<'_, ApplicationState>) -> Result<(), PtyResponseError> {
    let thread_map = Arc::clone(&state.pty_thread_map);
    let mut threads = thread_map.lock().map_err(|e| {
        PtyError::InternalError(format!(
            "Error occurred obtaining lock to instaces map.\n{:?}",
            e
        ))
    })?;

    let pty_thread = threads
        .get_mut(&instance_id)
        .ok_or(PtyError::WriteError(format!(
            "Instance with id {} not found.",
            &instance_id
        )))?;

    let read_tx = &pty_thread.pty_read_tx;
    let write_tx = &pty_thread.pty_write_tx;

    // The read interrupt is sent first because BufReader is blocking.
    // The write interrupt will send an EOF which will unblock the read thread, which will then get the interrupt message.

    read_tx.send(PtyMessage::Interrupt).map_err(|e| {
        PtyError::InternalError(format!(
            "Error occurred transmitting interrupt to instance thread.\n{:?}",
            e
        ))
    })?;

    write_tx.send(PtyMessage::Interrupt).map_err(|e| {
        PtyError::InternalError(format!(
            "Error occurred transmitting interrupt to instance thread.\n{:?}",
            e
        ))
    })?;

    pty_thread.join()?;

    threads.remove(&instance_id);

    Ok(())
}

#[tauri::command]
fn resize(
    instance_id: u32,
    rows: u16,
    cols: u16,
    state: State<'_, ApplicationState>,
) -> Result<(), PtyResponseError> {
    let thread_map = Arc::clone(&state.pty_thread_map);
    let mut threads = thread_map.lock().map_err(|e| {
        PtyError::InternalError(format!(
            "Error occurred obtaining lock to instaces map.\n{:?}",
            e
        ))
    })?;

    let pty_thread = threads
        .get_mut(&instance_id)
        .ok_or(PtyError::WriteError(format!(
            "Instance with id {} not found.",
            &instance_id
        )))?;

    let read_tx = &pty_thread.pty_read_tx;
    let write_tx = &pty_thread.pty_write_tx;

    read_tx.send(PtyMessage::Resize(rows, cols)).map_err(|e| {
        PtyError::InternalError(format!(
            "Error occurred transmitting resize command to instance thread.\n{:?}",
            e
        ))
    })?;

    Ok(())
}
fn main() {
    let application_state = ApplicationState::create().unwrap();
    tauri::Builder::default()
        .manage(application_state)
        .invoke_handler(tauri::generate_handler![create, write, destroy, resize])
        .run(tauri::generate_context!())
        .expect("Error occurred while running tauri application.");
}
