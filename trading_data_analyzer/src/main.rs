pub mod server;
pub mod stats;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    server::start_server().await
}
