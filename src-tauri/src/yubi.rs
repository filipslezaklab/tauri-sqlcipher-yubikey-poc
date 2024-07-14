use std::{
  fs::{File, OpenOptions},
  io::{Read, Write},
  path::PathBuf,
  str::FromStr,
  time::Duration,
};

use crate::{
  protector::Protector,
  state::{AppState, PasswordProtector},
};
use passwords::PasswordGenerator;
use rsa::{
  pkcs1v15::EncryptingKey,
  pkcs8::{der::Encode, DecodePublicKey},
  traits::RandomizedEncryptor,
  RsaPublicKey,
};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, State};
use tokio::time::sleep;
use yubikey::{
  piv::{self, SlotId},
  Context, Serial, YubiKey,
};

#[derive(Debug, Serialize, Deserialize, Type)]
pub struct SelectedYubiKey {
  pub serial: String,
  pub pin: String,
}

#[derive(Debug, Default)]
pub struct YubiKeyManager {
  password: Option<String>,
}

impl Protector for YubiKeyManager {
  async fn encrypt(
    &mut self,
    app: &AppHandle,
    manager: &SelectedYubiKey,
    password: String,
  ) -> Result<(), String> {
    let mut pass_file = OpenOptions::new()
      .create(true)
      .write(true)
      .open(format!("./{}", &manager.serial))
      .unwrap();
    let serial = Serial::from_str(&manager.serial).unwrap();
    let mut yubikey = YubiKey::open_by_serial(serial)
      .map_err(|e| e.to_string())
      .unwrap();
    yubikey.verify_pin(manager.pin.as_bytes()).unwrap();
    let target_slot = SlotId::KeyManagement;
    let slot_meta = piv::metadata(&mut yubikey, target_slot).unwrap();
    let key_info = slot_meta.public.unwrap();
    let key_der = &key_info.to_der().unwrap();
    let public_key = RsaPublicKey::from_public_key_der(&key_der).unwrap();
    let pkcs1v15 = EncryptingKey::new(public_key);
    println!("{:?}", &password);
    let data_bytes = password.as_bytes();
    let encrypted_data = pkcs1v15
      .encrypt_with_rng(&mut rand::thread_rng(), data_bytes)
      .unwrap();
    pass_file.write_all(&encrypted_data).unwrap();
    self.password = Some(password.clone());
    Ok(())
  }

  async fn decrypt(
    &mut self,
    app: &AppHandle,
    manager: &SelectedYubiKey,
  ) -> Result<String, String> {
    let mut pass_file = OpenOptions::new()
      .read(true)
      .open(format!("./{}", &manager.serial))
      .unwrap();
    let mut file_buf: Vec<u8> = Vec::new();
    pass_file.read_to_end(&mut file_buf).unwrap();

    let serial = Serial::from_str(&manager.serial).unwrap();

    let mut yubikey = YubiKey::open_by_serial(serial).unwrap();
    yubikey.verify_pin(manager.pin.as_bytes()).unwrap();
    let decrypted_data = piv::decrypt_data(
      &mut yubikey,
      &file_buf,
      piv::AlgorithmId::Rsa2048,
      SlotId::KeyManagement,
    )
    .unwrap();
    // Removing PKCS1 v1.5 padding manually
    if decrypted_data.len() < 11 || decrypted_data[0] != 0x00 || decrypted_data[1] != 0x02 {
      return Err("Decryption error: invalid PKCS1 v1.5 padding".into());
    }

    let mut i = 2;
    while i < decrypted_data.len() && decrypted_data[i] != 0 {
      i += 1;
    }
    if i >= decrypted_data.len() {
      return Err("Decryption error: invalid PKCS1 v1.5 padding".into());
    }

    let decrypted_content = &decrypted_data[(i + 1)..];
    let lossy_res = String::from_utf8_lossy(decrypted_content).to_string();
    Ok(lossy_res)
  }

  async fn get_password(
    &mut self,
    app: &AppHandle,
    manager: &SelectedYubiKey,
  ) -> Result<String, String> {
    match &self.password {
      Some(pass) => Ok(pass.clone()),
      None => self.decrypt(app, manager).await,
    }
  }
}

// lists available yubikeys by serial number
#[tauri::command]
#[specta::specta]
pub async fn list_yk() -> Result<Vec<String>, String> {
  let mut connected_serials: Vec<String> = vec![];
  let mut started = false;

  while connected_serials.len() == 0 {
    if started {
      sleep(Duration::from_secs(4)).await;
    } else {
      started = true;
    }

    let mut context = Context::open().map_err(|e| e.to_string())?;
    let readers = context.iter().map_err(|e| e.to_string())?;

    for reader in readers {
      let mut yubikey = reader.open().map_err(|e| e.to_string())?;
      let serial = yubikey.serial();
      let retries = yubikey.get_pin_retries().unwrap();
      if retries == 0 {
        continue;
      }
      connected_serials.push(serial.clone().to_string());
    }
  }

  Ok(connected_serials)
}

// select the yubikey
#[tauri::command]
#[specta::specta]
pub async fn select_yubikey(
  state: State<'_, AppState>,
  data: SelectedYubiKey,
) -> Result<(), String> {
  // validate pin and serial
  let serial = Serial::from_str(&data.serial).map_err(|e| e.to_string())?;
  let mut yubikey = YubiKey::open_by_serial(serial).map_err(|e| e.to_string())?;
  yubikey
    .verify_pin(data.pin.as_bytes())
    .map_err(|e| e.to_string())?;
  // set value in mutex
  *state.selected_yubikey.lock().await = Some(data);
  *state.protector.lock().await = Some(PasswordProtector::YubiKey);
  Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn verify_yubikey_pin(data: SelectedYubiKey) -> Result<(), i32> {
  let serial = Serial::from_str(&data.serial).unwrap();
  let mut yubikey = YubiKey::open_by_serial(serial).unwrap();
  let verification = yubikey.verify_pin(data.pin.as_bytes());
  match verification {
    Ok(_) => Ok(()),
    Err(_e) => {
      let retries = yubikey.get_pin_retries().unwrap();
      return Err(retries as i32);
    }
  }
}

pub async fn protect_password(
  protector: &PasswordProtector,
  yubikey_info: &SelectedYubiKey,
  app: &AppHandle,
) -> Result<(), String> {
  match protector {
    PasswordProtector::YubiKey => {
      let mut manager = YubiKeyManager::default();
      let pg = PasswordGenerator {
        length: 128,
        numbers: true,
        lowercase_letters: true,
        uppercase_letters: true,
        symbols: false,
        spaces: false,
        exclude_similar_characters: false,
        strict: true,
      };
      let password = pg.generate_one().map_err(|e| e.to_string())?;
      manager.encrypt(&app, yubikey_info, password).await?;
    }
    _ => {
      return Err("Protector not implemented".into());
    }
  }
  Ok(())
}
