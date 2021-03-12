use actix_web::{web, App, HttpRequest, HttpServer, Responder, HttpResponse};
use anyhow::Result;
use packer::Packer;

use crate::config::Config;

#[derive(Packer)]
#[packer(source = "static")]
struct Statics;

#[derive(Packer)]
#[packer(source = "templates")]
struct Templates;

pub async fn run_web(config: Config) -> Result<()> {
    HttpServer::new(|| App::new().route("/", web::get().to(index)))
        .bind((config.host.as_str(), config.port))?
        .run()
        .await?;

    Ok(())
}

async fn index(req: HttpRequest) -> impl Responder {
    HttpResponse::Ok().body("under construction")
}
