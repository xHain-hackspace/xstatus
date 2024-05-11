mod api;
mod errors;
mod settings;

use actix_web::{middleware::Logger, web::Data, App, HttpServer};
use spaceapi::Status;
use std::sync::Mutex;

struct AppState {
    status: Mutex<spaceapi::Status>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Read the config
    let config_file = std::env::var("XSTATUS_CONFIG_FILE").unwrap_or("config.toml".to_string());
    let settings = settings::Settings::new(config_file.as_str()).unwrap();

    std::env::set_var("RUST_LOG", "debug");
    std::env::set_var("RUST_BACKTRACE", "1");
    pretty_env_logger::init();

    log::info!("Listening on {}", settings.endpoint);

    let status = Status { ..settings.status };

    let app_state_data = Data::new(AppState {
        status: Mutex::new(status),
    });
    HttpServer::new(move || {
        let logger = Logger::default();
        App::new()
            .wrap(logger)
            .service(api::get_status)
            .service(api::set_state)
            .app_data(app_state_data.clone())
    })
    .bind(settings.endpoint)?
    .run()
    .await
}
