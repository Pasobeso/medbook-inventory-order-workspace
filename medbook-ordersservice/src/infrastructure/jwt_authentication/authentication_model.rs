use serde::{Deserialize, Serialize};

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct LoginModel {
    pub hospital_number: i32,
    pub password: String,
}