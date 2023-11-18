// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod state;

use state::ApplicationState;
use tauri::State;

#[tauri::command]
fn write(
    instance_id: u32,
    input: String,
    state: State<'_, ApplicationState>,
) -> Result<(), String> {
    let mut instances = state.instances.lock().unwrap();
    let instance = instances
        .get_mut(&instance_id)
        .ok_or("Invalid instance ID.")?;
    instance.write(input).map_err(|e| format!("{:?}", e))
}

#[tauri::command]
fn read(instance_id: u32, state: State<'_, ApplicationState>) -> Result<String, String> {
    let mut instances = state.instances.lock().unwrap();
    let instance = instances
        .get_mut(&instance_id)
        .ok_or("Invalid instance ID.")?;
    instance.read().map_err(|e| format!("{:?}", e))
}

fn main() {
    let application_state = ApplicationState::create().unwrap();
    tauri::Builder::default()
        .manage(application_state)
        .invoke_handler(tauri::generate_handler![read, write])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
