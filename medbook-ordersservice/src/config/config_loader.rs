use anyhow::Result;

use super::{config_model::{ PatientsSecret, Database, DotEnvyConfig, DoctorsSecret, Server}, stage::Stage};

pub fn load() -> Result<DotEnvyConfig> {
    dotenvy::dotenv().ok();

    let server = Server {
        port: std::env::var("SERVER_PORT")
            .expect("SERVER_PORT is invalid")
            .parse()?,
        body_limit: std::env::var("SERVER_BODY_LIMIT")
            .expect("SERVER_BODY_LIMIT is invalid")
            .parse()?,
        timeout: std::env::var("SERVER_TIMEOUT")
            .expect("SERVER_TIMEOUT is invalid")
            .parse()?,
    };

    let database = Database{
        url: std::env::var("DATABASE_URL").expect("DATABASE_URL is invalid"),
    };

    Ok(DotEnvyConfig {server,database})
}

pub fn get_stage() -> Stage {
    dotenvy::dotenv().ok();

    let stage_str = std::env::var("STAGE").unwrap_or("".to_string());
    Stage::try_from(&stage_str).unwrap_or_default()
}

pub fn get_patients_secret_env() -> Result<PatientsSecret> {
    dotenvy::dotenv().ok();

    Ok(PatientsSecret {
        secret: std::env::var("JWT_PATIENT_SECRET").expect("JWT_PATIENT_SECRET is invalid"),
        refresh_secret: std::env::var("JWT_PATIENT_REFRESH_SECRET").expect("JWT_PATIENT_REFRESH_SECRET is invalid"),

    })
}

pub fn get_doctors_secret_env() -> Result<DoctorsSecret> {
    dotenvy::dotenv().ok();

    Ok(DoctorsSecret {
        secret: std::env::var("JWT_DOCTOR_SECRET").expect("JWT_DOCTOR_SECRET is invalid"),
        refresh_secret: std::env::var("JWT_DOCTOR_REFRESH_SECRET").expect("JWT_DOCTOR_REFRESH_SECRET is invalid"),

    })
}
