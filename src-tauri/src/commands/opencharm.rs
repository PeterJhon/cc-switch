//! OpenCharm process controls are desktop-only Tauri IPC, not remote APIs.

use crate::opencharm::{self, RuntimeManager, RuntimeStatus};
use std::sync::Arc;
use tauri::{AppHandle, Manager, State};

fn paths(app: &AppHandle) -> Result<(std::path::PathBuf, std::path::PathBuf), String> {
    let res = app.path().resource_dir().map_err(|e| e.to_string())?;
    let data = app.path().app_local_data_dir().map_err(|e| e.to_string())?;
    Ok((opencharm::executable(&res), data.join("opencharm")))
}

#[tauri::command]
pub fn get_opencharm_status(app: AppHandle, runtime: State<'_, Arc<RuntimeManager>>) -> Result<RuntimeStatus, String> {
    let (binary, _) = paths(&app)?;
    runtime.status(&binary)
}

#[tauri::command]
pub async fn start_opencharm(app: AppHandle, runtime: State<'_, Arc<RuntimeManager>>) -> Result<RuntimeStatus, String> {
    let (binary, data) = paths(&app)?;
    let runtime = Arc::clone(&*runtime);
    tauri::async_runtime::spawn_blocking(move || runtime.start(&binary, &data))
        .await.map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn stop_opencharm(runtime: State<'_, Arc<RuntimeManager>>) -> Result<(), String> {
    let runtime = Arc::clone(&*runtime);
    tauri::async_runtime::spawn_blocking(move || runtime.stop())
        .await.map_err(|e| e.to_string())?
}
