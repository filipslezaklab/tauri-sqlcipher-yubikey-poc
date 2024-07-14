use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use sqlx::SqlitePool;

use crate::yubi::SelectedYubiKey;

#[derive(Debug, Default)]
pub struct AppState {
  pub selected_yubikey: Mutex<Option<SelectedYubiKey>>,
  pub pool: Mutex<Option<SqlitePool>>,
  pub protector: Mutex<Option<PasswordProtector>>,
}

#[derive(strum_macros::Display, Serialize, Deserialize, Debug)]
pub enum PasswordProtector {
  #[strum(to_string = "yubikey")]
  YubiKey,
  #[strum(to_string = "unset")]
  Unset,
}
