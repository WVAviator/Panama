// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod state;

use state::ApplicationState;
use tauri::State;

#[derive(serde::Serialize)]
struct WriteResponse {
    instance_id: u32,
    output: String,
    error: Option<String>,
}

#[tauri::command]
fn write(
    instance_id: u32,
    input: String,
    state: State<'_, ApplicationState>,
) -> Result<WriteResponse, WriteResponse> {
    let mut instances = state.instances.lock().unwrap();
    let instance = instances.get_mut(&instance_id).ok_or(WriteResponse {
        instance_id,
        output: String::new(),
        error: Some(format!("Instance with ID {} does not exist.", instance_id)),
    })?;
    let output = instance.write(input).map_err(|e| WriteResponse {
        instance_id,
        output: String::new(),
        error: Some(format!("{:?}", e)),
    })?;

    Ok(WriteResponse {
        instance_id,
        output,
        error: None,
    })
}

#[derive(serde::Serialize)]
struct ReadResponse {
    instance_id: u32,
    output: String,
    error: Option<String>,
}

#[tauri::command]
fn read(
    instance_id: u32,
    state: State<'_, ApplicationState>,
) -> Result<ReadResponse, ReadResponse> {
    println!("Command read called");
    let mut instances = state.instances.lock().unwrap();
    let instance = instances.get_mut(&instance_id).ok_or(ReadResponse {
        instance_id,
        output: String::new(),
        error: Some("Invalid instance ID.".to_string()),
    })?;

    let output = instance.read().map_err(|e| ReadResponse {
        instance_id,
        output: String::new(),
        error: Some(format!("{:?}", e)),
    })?;

    println!("Output: {}", output);

    Ok(ReadResponse {
        instance_id,
        output,
        error: None,
    })
}

fn main() {
    let application_state = ApplicationState::create().unwrap();
    tauri::Builder::default()
        .manage(application_state)
        .invoke_handler(tauri::generate_handler![read, write])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
