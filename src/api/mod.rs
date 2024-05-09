use crate::AppState;
use actix_web::{error, get, post, web::Data, web::Form, web::Json};
use serde::Deserialize;
use spaceapi::{State, Status};

use derive_more::{Display, Error};

#[derive(Debug, Display, Error)]
#[display(fmt = "error setting the state: {}", reason)]
struct SetStateError {
    reason: String,
}

// Use default implementation for `error_response()` method
impl error::ResponseError for SetStateError {}

#[get("/status")]
pub async fn get_status(app_state: Data<AppState>) -> Json<Status> {
    let status = app_state.status.lock().unwrap().clone();
    Json(status)
}

#[derive(Debug, Deserialize)]
struct StateFormData {
    open: Option<bool>,
    message: Option<String>,
}

#[post("/status/state")]
pub async fn set_state(
    app_state: Data<AppState>,
    new_state_data: Form<StateFormData>,
) -> Result<Json<String>, SetStateError> {
    let mut status = match app_state.status.lock() {
        Err(err) => {
            log::error!("unable get app status: {}", err);
            return Err(SetStateError {
                reason: err.to_string(),
            });
        }
        Ok(status) => status,
    };
    let mut default_state = State {
        ..Default::default()
    };
    let state: &mut spaceapi::State = match &mut status.state {
        None => &mut default_state,
        Some(state) => state,
    };
    state.open = new_state_data.open;
    state.message.clone_from(&new_state_data.message);
    status.state = Some(state.to_owned());
    Ok(Json(String::from("Success")))
}
