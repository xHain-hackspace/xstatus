use actix_web::{body::BoxBody, http::StatusCode, HttpResponse, ResponseError};
use serde::Serialize;
use std::fmt;

#[derive(Debug)]
pub enum AppErrorType {
    InternalError,
    NotFoundError,
}

#[derive(Debug)]
pub struct AppError {
    pub message: Option<String>,
    pub cause: Option<String>,
    pub error_type: AppErrorType,
}

impl AppError {
    fn message(&self) -> String {
        match self {
            AppError {
                message: Some(message),
                cause: _,
                error_type: _,
            } => message.clone(),
            AppError {
                message: None,
                cause: _,
                error_type: AppErrorType::NotFoundError,
            } => "The response item was not found".to_string(),
            _ => "An unexpected error has occured".to_string(),
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Serialize)]
pub struct AppErrorResponse {
    pub error: String,
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self.error_type {
            AppErrorType::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            AppErrorType::NotFoundError => StatusCode::NOT_FOUND,
        }
    }

    fn error_response(&self) -> actix_web::HttpResponse<BoxBody> {
        if let Some(cause) = &self.cause {
            log::error!("{}", cause);
        }
        HttpResponse::build(self.status_code()).json(AppErrorResponse {
            error: self.message(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::body::to_bytes;
    use serde_json::Value;

    fn error(message: Option<&str>, error_type: AppErrorType) -> AppError {
        AppError {
            message: message.map(String::from),
            cause: Some("cause".to_string()),
            error_type,
        }
    }

    async fn response_body(err: &AppError) -> Value {
        let body = to_bytes(err.error_response().into_body()).await.unwrap();
        serde_json::from_slice(&body).unwrap()
    }

    #[test]
    fn status_codes() {
        assert_eq!(
            error(None, AppErrorType::InternalError).status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
        assert_eq!(
            error(None, AppErrorType::NotFoundError).status_code(),
            StatusCode::NOT_FOUND
        );
    }

    #[actix_web::test]
    async fn default_messages() {
        let not_found = error(None, AppErrorType::NotFoundError);
        assert_eq!(
            response_body(&not_found).await["error"],
            "The response item was not found"
        );

        let internal = error(None, AppErrorType::InternalError);
        assert_eq!(
            response_body(&internal).await["error"],
            "An unexpected error has occured"
        );
    }

    #[actix_web::test]
    async fn custom_message_takes_precedence() {
        let err = error(Some("custom"), AppErrorType::NotFoundError);
        assert_eq!(err.error_response().status(), StatusCode::NOT_FOUND);
        assert_eq!(response_body(&err).await["error"], "custom");
    }
}
