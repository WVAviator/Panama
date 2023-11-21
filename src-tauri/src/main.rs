// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod pty_response_error;
mod state;

use std::{
    io::{BufRead, BufReader},
    sync::Arc,
};

use pty_response_error::PtyResponseError;
use state::{pty_instance::PtyInstance, ApplicationState};
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
    window: Window,
    state: State<'_, ApplicationState>,
) -> Result<CreateResponse, PtyResponseError> {
    let instance_id = rand::random::<u32>();
    println!("Creating pty instance with id: {}", instance_id);

    let (tx, rx) = std::sync::mpsc::channel();
    let instances_ref = Arc::clone(&state.pty_write_tx_map);
    {
        let mut instances = instances_ref.lock().unwrap();
        instances.insert(instance_id, tx);
    }

    std::thread::spawn(move || {
        let instance = PtyInstance::create(rows, cols).unwrap();
        let mut writer = instance.writer;

        std::thread::spawn(move || loop {
            let write = rx.recv().unwrap();
            writer
                .write_all(write.as_bytes())
                .expect("Unable to write to instance");
        });

        loop {
            let reader = instance.pty_pair.master.try_clone_reader().unwrap();
            let mut buf_reader = BufReader::new(reader);

            let data = buf_reader.fill_buf().unwrap();
            let len = data.len();

            let data = std::str::from_utf8(data).unwrap().to_string();
            buf_reader.consume(len);

            window
                .emit(
                    format!("read:{}", instance_id).as_str(),
                    ReadResponse { output: data },
                )
                .unwrap();
        }
    });

    Ok(CreateResponse { instance_id })
}

#[tauri::command]
fn write(
    instance_id: u32,
    input: String,
    state: State<'_, ApplicationState>,
) -> Result<(), PtyResponseError> {
    let instances_ref = Arc::clone(&state.pty_write_tx_map);
    let mut instances = instances_ref.lock().unwrap();
    let tx = instances.get_mut(&instance_id).unwrap();
    tx.send(input).unwrap();

    Ok(())
}

fn main() {
    let application_state = ApplicationState::create().unwrap();
    tauri::Builder::default()
        .manage(application_state)
        .invoke_handler(tauri::generate_handler![create, write])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
