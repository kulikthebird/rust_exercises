use crate::stats::*;
use actix_web::{App, HttpResponse, HttpServer, Responder, get, post, web};
use lazy_static::lazy_static;
use serde::Deserialize;
use std::collections::HashMap;
use tokio::sync::RwLock;

lazy_static! {
    static ref DATA_STORAGE: RwLock<HashMap<String, Symbol>> = RwLock::new(HashMap::new());
}

#[derive(Deserialize)]
struct AddBatchRequest {
    symbol: String,
    values: Vec<f64>,
}

#[post("/add_batch/")]
async fn add_batch(req: web::Json<AddBatchRequest>) -> impl Responder {
    let mut storage = DATA_STORAGE.write().await;
    let entry = storage
        .entry(req.symbol.clone())
        .or_insert(Symbol::default());
    entry.insert(&req.values);
    HttpResponse::Ok().json("Batch added successfully")
}

#[get("/stats/")]
async fn get_stats(query: web::Query<HashMap<String, String>>) -> impl Responder {
    let symbol = match query.get("symbol") {
        Some(s) => s.clone(),
        None => return HttpResponse::BadRequest().json("Missing symbol parameter"),
    };

    let k: usize = match query.get("k").and_then(|k| k.parse().ok()) {
        Some(k) if (1..=8).contains(&k) => k,
        _ => {
            return HttpResponse::BadRequest()
                .json("Invalid k parameter - should be value in segment [1, 8]");
        }
    };

    let storage = DATA_STORAGE.read().await;
    match storage.get(&symbol) {
        Some(d) => {
            if let Some(stats) = d.get_stats(k - 1) {
                HttpResponse::Ok().json(stats.get_stats_resp())
            } else {
                HttpResponse::NotFound().json("Window size too big")
            }
        }
        _ => HttpResponse::NotFound().json("Symbol not found or no data"),
    }
}

pub async fn start_server() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(add_batch).service(get_stats))
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
