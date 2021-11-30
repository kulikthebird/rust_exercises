/// Author: Tomasz Kulik
/// 
///

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    println!("Trading market");
    trading::start_server("127.0.0.1:8080".to_string()).await
}
