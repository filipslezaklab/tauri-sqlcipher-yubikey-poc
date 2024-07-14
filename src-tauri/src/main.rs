// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod db;
mod models;
mod protector;
mod state;
mod yubi;

use anyhow::Ok as AnyOk;
use anyhow::Result as AnyResult;
use db::{add_db_item, init_database, list_db};
use specta::collect_types;
use state::AppState;
use state::PasswordProtector;
use tauri::Manager;
use tauri::State;
use yubi::list_yk;
use yubi::select_yubikey;
use yubi::verify_yubikey_pin;

#[tokio::main]
async fn main() -> AnyResult<()> {
  tauri_specta::ts::export(
    collect_types![
      list_yk,
      select_yubikey,
      verify_yubikey_pin,
      init_database,
      list_db,
      add_db_item,
    ],
    "../src/bindings.ts",
  )
  .unwrap();

  let app = tauri::Builder::default()
    .manage(AppState::default())
    .invoke_handler(tauri::generate_handler![
      list_yk,
      select_yubikey,
      verify_yubikey_pin,
      init_database,
      list_db,
      add_db_item,
    ])
    .build(tauri::generate_context!())?;
  let app_handle = app.handle();
  let state: State<'_, AppState> = app_handle.state();
  *state.protector.lock().await = Some(PasswordProtector::Unset);
  app.run(|_, _| {});
  AnyOk(())
}
