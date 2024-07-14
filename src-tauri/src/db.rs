use std::path::Path;

use sqlx::{
  migrate::{MigrateError, Migrator},
  pool::PoolConnection,
  sqlite::{SqliteConnectOptions, SqlitePoolOptions},
  Pool, Sqlite, SqlitePool,
};
use tauri::{App, AppHandle, State};

use crate::{
  models::{MyField, MyFieldTs},
  protector::Protector,
  state::{AppState, PasswordProtector},
  yubi::{protect_password, YubiKeyManager},
};

pub static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

pub async fn run_migrations(pool: &Pool<Sqlite>) -> Result<(), MigrateError> {
  MIGRATOR.run(pool).await
}

pub async fn make_pool(app: &AppHandle, password: String) -> Result<SqlitePool, sqlx::Error> {
  let database_path = Path::new("./data.db");
  let options = SqliteConnectOptions::new()
    .filename(database_path)
    .create_if_missing(true)
    .pragma("key", password.clone())
    .pragma("journal_mode", "Delete");
  let pool = SqlitePoolOptions::new().connect_with(options).await?;
  Ok(pool)
}

pub async fn insert_record(pool: &Pool<Sqlite>, value: String) -> Result<(), String> {
  sqlx::query!("INSERT INTO test (test_field) VALUES (?)", value)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;
  Ok(())
}

pub async fn get_connection(
  state: State<'_, AppState>,
) -> Result<PoolConnection<Sqlite>, sqlx::Error> {
  let pool_guard = state.pool.lock().await;
  if let Some(ref pool) = *pool_guard {
    return Ok(pool.acquire().await?);
  }

  Err(sqlx::Error::PoolTimedOut)
}

#[tauri::command]
#[specta::specta]
pub async fn list_db(state: State<'_, AppState>) -> Result<Vec<MyFieldTs>, String> {
  match *state.pool.lock().await {
    Some(ref pool) => {
      let records: Vec<MyField> = sqlx::query_as!(MyField, "SELECT * FROM test;")
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;
      let res: Vec<MyFieldTs> = records.iter().map(|item| MyFieldTs::from(item)).collect();
      Ok(res)
    }
    None => Err("No Pool".into()),
  }
}

#[tauri::command]
#[specta::specta]
pub async fn add_db_item(state: State<'_, AppState>, data: MyFieldTs) -> Result<(), String> {
  match *state.pool.lock().await {
    Some(ref pool) => insert_record(pool, data.test_field).await,
    None => Err("No Pool".into()),
  }
}

#[tauri::command]
#[specta::specta]
pub async fn init_database(state: State<'_, AppState>, app: AppHandle) -> Result<(), String> {
  println!("Attempt to init database");
  if let Some(ref protector) = *state.protector.lock().await {
    match *protector {
      PasswordProtector::YubiKey => {
        if let Some(ref yubikey_info) = *state.selected_yubikey.lock().await {
          let mut manager = YubiKeyManager::default();
          let file_path = format!("./{}", &yubikey_info.serial);
          if !Path::new(&file_path).exists() {
            println!("Pass file not found for serial {}", yubikey_info.serial);
            protect_password(protector, yubikey_info, &app).await?;
            println!("Password for the key was generated.");
          }
          let password = manager.decrypt(&app, yubikey_info).await?;
          let pool = make_pool(&app, password).await.map_err(|e| e.to_string())?;
          run_migrations(&pool).await.map_err(|e| e.to_string())?;
          *state.pool.lock().await = Some(pool);
          return Ok(());
        }
        Err("Yubikey not selected.".into())
      }
      PasswordProtector::Unset => Err("Password protector is unset.".into()),
    }
  } else {
    Err("Init failed".into())
  }
}
