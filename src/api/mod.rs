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
                .unwrap_or("false".to_string())
                .to_lowercase()
                .parse::<bool>()
                .map_err(|err| AppError {
                    message: None,
                    cause: Some(err.to_string()),
                    error_type: AppErrorType::InternalError,
                });
            Some(parsed_open?)
        }
        Some(open) => Some(open),
    };
    state.message.clone_from(&new_state_data.message);
    status.state = Some(state.to_owned());
    Ok(Json(String::from("Success")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{dev::ServiceResponse, http::StatusCode, test, App};
    use serde_json::{json, Value};
    use std::sync::Mutex;

    fn app_state(status: Status) -> Data<AppState> {
        Data::new(AppState {
            status: Mutex::new(status),
        })
    }

    fn test_status() -> Status {
        Status {
            space: "Test Space".to_string(),
            ..Default::default()
        }
    }

    async fn post_state(state: &Data<AppState>, body: Value) -> ServiceResponse {
        let app = test::init_service(App::new().app_data(state.clone()).service(set_state)).await;
        let req = test::TestRequest::post()
            .uri("/status/state")
            .set_json(body)
            .to_request();
        test::call_service(&app, req).await
    }

    fn current_state(state: &Data<AppState>) -> State {
        state.status.lock().unwrap().state.clone().unwrap()
    }

    #[actix_web::test]
    async fn get_status_returns_current_status() {
        let state = app_state(test_status());
        let app = test::init_service(App::new().app_data(state.clone()).service(get_status)).await;

        let req = test::TestRequest::get().uri("/status").to_request();
        let status: Status = test::call_and_read_body_json(&app, req).await;

        assert_eq!(status, test_status());
    }

    #[actix_web::test]
    async fn set_state_with_bool() {
        let state = app_state(test_status());

        let resp = post_state(&state, json!({"open": true, "message": "come in"})).await;

        assert_eq!(resp.status(), StatusCode::OK);
        let body: String = test::read_body_json(resp).await;
        assert_eq!(body, "Success");
        let new_state = current_state(&state);
        assert_eq!(new_state.open, Some(true));
        assert_eq!(new_state.message.as_deref(), Some("come in"));
    }

    #[actix_web::test]
    async fn set_state_with_string_is_case_insensitive() {
        let state = app_state(test_status());

        for (open_string, expected) in [("true", true), ("TRUE", true), ("False", false)] {
            let resp = post_state(&state, json!({ "open_string": open_string })).await;
            assert_eq!(resp.status(), StatusCode::OK, "open_string {open_string}");
            assert_eq!(
                current_state(&state).open,
                Some(expected),
                "open_string {open_string}"
            );
        }
    }

    #[actix_web::test]
    async fn set_state_prefers_bool_over_string() {
        let state = app_state(test_status());

        post_state(&state, json!({"open": false, "open_string": "true"})).await;

        assert_eq!(current_state(&state).open, Some(false));
    }

    #[actix_web::test]
    async fn set_state_defaults_to_closed() {
        let state = app_state(test_status());

        let resp = post_state(&state, json!({})).await;

        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(current_state(&state).open, Some(false));
    }

    #[actix_web::test]
    async fn set_state_clears_message_when_omitted() {
        let state = app_state(test_status());
        post_state(&state, json!({"open": true, "message": "come in"})).await;

        post_state(&state, json!({"open": false})).await;

        let new_state = current_state(&state);
        assert_eq!(new_state.open, Some(false));
        assert_eq!(new_state.message, None);
    }

    #[actix_web::test]
    async fn set_state_rejects_unparseable_string_and_keeps_state() {
        let state = app_state(test_status());
        post_state(&state, json!({"open": true})).await;

        let resp = post_state(&state, json!({"open_string": "yes"})).await;

        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(current_state(&state).open, Some(true));
        assert!(!state.status.is_poisoned());
    }

    #[actix_web::test]
    async fn set_state_rejects_invalid_json() {
        let state = app_state(test_status());
        let app = test::init_service(App::new().app_data(state.clone()).service(set_state)).await;

        let req = test::TestRequest::post()
            .uri("/status/state")
            .insert_header(("content-type", "application/json"))
            .set_payload("not json")
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        assert_eq!(state.status.lock().unwrap().state, None);
    }

    #[actix_web::test]
    async fn get_status_reflects_set_state() {
        let state = app_state(test_status());
        let app = test::init_service(
            App::new()
                .app_data(state.clone())
                .service(get_status)
                .service(set_state),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/status/state")
            .set_json(json!({"open": true, "message": "open"}))
            .to_request();
        test::call_service(&app, req).await;
        let req = test::TestRequest::get().uri("/status").to_request();
        let status: Status = test::call_and_read_body_json(&app, req).await;

        let state = status.state.unwrap();
        assert_eq!(state.open, Some(true));
        assert_eq!(state.message.as_deref(), Some("open"));
    }
}
