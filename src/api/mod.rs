use crate::errors::{AppError, AppErrorType};
use crate::AppState;
use actix_web::{get, post, web::Data, web::Json, Result};
use serde::Deserialize;
use spaceapi::{State, Status};

#[get("/status")]
pub async fn get_status(app_state: Data<AppState>) -> Result<Json<Status>> {
    let status = app_state.status.lock().map_err(|err| AppError {
        message: None,
        cause: Some(err.to_string()),
        error_type: AppErrorType::NotFoundError,
    })?;

    Ok(Json(status.clone()))
}

#[derive(Debug, Deserialize)]
struct StateData {
    open: Option<bool>,
    open_string: Option<String>,
    message: Option<String>,
}

#[post("/status/state")]
pub async fn set_state(
    app_state: Data<AppState>,
    new_state_data: Json<StateData>,
) -> Result<Json<String>> {
    let mut status = app_state.status.lock().map_err(|err| AppError {
        message: None,
        cause: Some(err.to_string()),
        error_type: AppErrorType::InternalError,
    })?;

    let mut default_state = State {
        ..Default::default()
    };
    let state: &mut spaceapi::State = match &mut status.state {
        None => &mut default_state,
        Some(state) => state,
    };
    state.open = match new_state_data.open {
        None => {
            // try to parse the string version
            let parsed_open = new_state_data
                .open_string
                .to_owned()
                .unwrap()
                .to_lowercase()
                .parse::<bool>()
                .map_err(|err| AppError {
                    message: None,
                    cause: Some(format!(
                        "error parsing open_string: {} for: {}",
                        err,
                        new_state_data.open_string.clone().unwrap()
                    )),
                    error_type: AppErrorType::InternalError,
                });
            Some(parsed_open.unwrap())
        }
        Some(open) => Some(open),
    };
    state.message.clone_from(&new_state_data.message);
    status.state = Some(state.to_owned());
    Ok(Json(String::from("Success")))
}
