use serde::{Deserialize, Serialize};

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Passport {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Claims {
    pub sub: String,
    pub role: Roles,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Debug,Clone,Serialize,Deserialize,PartialEq)]
pub enum Roles {
    Patient,
    Doctor
}