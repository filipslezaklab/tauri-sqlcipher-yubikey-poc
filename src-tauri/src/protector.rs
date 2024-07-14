use tauri::{api::path::app_data_dir, App, AppHandle};

use crate::yubi::SelectedYubiKey;

pub trait Protector {
  async fn encrypt(
    &mut self,
    app: &AppHandle,
    manager: &SelectedYubiKey,
    password: String,
  ) -> Result<(), String>;
  async fn decrypt(&mut self, app: &AppHandle, manager: &SelectedYubiKey)
    -> Result<String, String>;
  async fn get_password(
    &mut self,
    app: &AppHandle,
    manager: &SelectedYubiKey,
  ) -> Result<String, String>;
}
