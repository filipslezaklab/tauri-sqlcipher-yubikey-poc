use serde::{Deserialize, Serialize};
use specta::Type;
use sqlx::prelude::FromRow;

#[derive(FromRow, Serialize, Deserialize)]
pub struct MyField {
  pub id: i64,
  pub test_field: String,
}

#[derive(Serialize, Deserialize, Type)]
pub struct MyFieldTs {
  pub id: Option<String>,
  pub test_field: String,
}

impl From<&MyField> for MyFieldTs {
  fn from(value: &MyField) -> Self {
    Self {
      id: Some(value.id.to_string()),
      test_field: value.test_field.clone(),
    }
  }
}
