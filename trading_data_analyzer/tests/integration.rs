use reqwest::Client;
use serde_json::json;
use std::process::{Child, Command};
use tokio::time::{Duration, sleep};

struct WrappedChild(Child);

impl Drop for WrappedChild {
    fn drop(&mut self) {
        self.0.kill().expect("Could not stop the server process")
    }
}

fn start_server() -> WrappedChild {
    WrappedChild(
        Command::new("cargo")
            .args(["run"])
            .spawn()
            .expect("Failed to start the server"),
    )
}

#[tokio::test]
async fn test_restful_api() {
    let _server = start_server();

    let client = Client::new();
    let base_url = "http://127.0.0.1:8080";

    // Step 1: Add batch data
    let symbol = "AAPL";
    let values = vec![100.0, 101.5, 102.3, 99.8, 100.5];
    let add_batch_url = format!("{}/add_batch/", base_url);

    // Try to send POST several times - the server might not be ready
    // from the start.
    let mut response = None;
    for _ in 0..20 {
        response = Some(
            client
                .post(&add_batch_url)
                .json(&json!({ "symbol": symbol, "values": values }))
                .send()
                .await,
        );

        if response.as_ref().unwrap().is_ok() {
            break;
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
    }

    assert!(
        response
            .unwrap()
            .expect("Failed to connect with the server")
            .status()
            .is_success()
    );

    // Allow time for the server to process the request
    sleep(Duration::from_millis(100)).await;

    // Step 2: Fetch statistics
    let k = 1; // last 10^1 = 10 data points
    let stats_url = format!("{}/stats/?symbol={}&k={}", base_url, symbol, k);

    let response = client
        .get(&stats_url)
        .send()
        .await
        .expect("Failed to send stats request");

    assert!(response.status().is_success());

    let body = response
        .json::<serde_json::Value>()
        .await
        .expect("Failed to parse JSON response");

    // Step 3: Validate response
    assert_eq!(body["last"], 100.5);
    assert_eq!(body["min"], 99.8);
    assert_eq!(body["max"], 102.3);
    assert!(body["avg"].as_f64().unwrap() > 100.0);
    assert!(body["var"].as_f64().unwrap() > 0.0);

    println!("Integration test passed!");
}
