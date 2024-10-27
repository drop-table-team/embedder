use actix_web::{
    post,
    web::{Data, Json},
    HttpResponse,
};
use log::error;

use crate::embedder::{Embedder, Input};

#[post("/queue")]
async fn queue(embedder: Data<&'static Embedder>, input: Json<Input>) -> HttpResponse {
    let input = input.into_inner();

    if let Err(e) = embedder.queue(input).await {
        error!("Couldn't queue input: {}", e);
        return HttpResponse::InternalServerError().finish();
    }

    HttpResponse::Ok().finish()
}
