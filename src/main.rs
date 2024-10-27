use std::env;

use actix_web::{web::Data, App, HttpServer};
use api::queue;
use embedder::Embedder;
use log::{error, info};
use serde::Deserialize;

pub mod api;
pub mod embedder;
pub mod ollama;

#[derive(Deserialize, Debug)]
struct Config {
    address: String,
    ollama_address: String,
    qdrant_address: String,
    qdrant_collection: String,
}

#[tokio::main]
async fn main() {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info")
    }

    env_logger::init();

    let config = match envy::from_env::<Config>() {
        Ok(c) => c,
        Err(e) => {
            error!("Couldn't parse environment variables: {}", e);
            return;
        }
    };

    info!("Loaded config: {:?}", config);

    let embedder = match Embedder::new(
        config.ollama_address,
        config.qdrant_address,
        config.qdrant_collection,
    )
    .await
    {
        Ok(e) => e,
        Err(e) => {
            return error!("Couldn't create Embedder: {}", e);
        }
    };

    let embedder: &'static Embedder = Box::leak(Box::new(embedder));

    embedder.start().await;

    HttpServer::new(move || App::new().service(queue).app_data(Data::new(embedder)))
        .bind(config.address)
        .unwrap()
        .run()
        .await
        .unwrap();
}
