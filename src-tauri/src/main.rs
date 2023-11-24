// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod pty_response_error;
mod state;

use std::sync::Arc;

use portable_pty::PtySize;
use pty_response_error::PtyResponseError;
use state::{pty_error::PtyError, pty_instance::PtyInstanceThread, ApplicationState};
use tauri::{State, Window};

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
        .get_mut(&instance_id)
    {
        println!("Instance with id {} already exists.", instance_id);

        pty_thread.resize(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        return Ok(CreateResponse { instance_id });
    }

    println!("Creating pty instance with id: {}", instance_id);

    let handler_fn = Box::new(move |output: String| {
        window
            .emit(
                format!("read:{}", instance_id).as_str(),
                ReadResponse { output },
            )
            .expect("Error occurred while emitting read event.");
    });

    let pty_instance_thread = PtyInstanceThread::new(
        PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        },
        &command,
        handler_fn,
    )?;

    {
        let mut thread_map = thread_map.lock().map_err(|e| {
            PtyError::InternalError(format!(
                "Error occurred while obtaining lock to threads map.\n{:?}",
                e
            ))
        })?;
        thread_map.insert(instance_id, pty_instance_thread);
    }

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
    pty_thread.write(input)?;

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

    pty_thread.close()?;

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

    pty_thread.resize(PtySize {
        rows,
        cols,
        pixel_width: 0,
        pixel_height: 0,
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
