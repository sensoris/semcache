use std::time::Duration;
use testcontainers::{
    GenericImage,
    core::{IntoContainerPort, WaitFor},
    runners::AsyncRunner,
};
use tokio::time::sleep;

const STARTUP_MESSAGE: &'static str = "Semcache started successfully";

#[tokio::main]
async fn main() {
    let container = GenericImage::new("semcache", "latest")
        .with_exposed_port(8080.tcp())
        .with_wait_for(WaitFor::message_on_stdout(STARTUP_MESSAGE))
        .start()
        .await
        .unwrap();

    let host = container.get_host().await.unwrap();
    let host_port = container.get_host_port_ipv4(8080).await.unwrap();

    // Wait for the server to start
    let url = format!("http://{}:{}/", host, host_port);

    let mut retries = 10;
    let client = reqwest::Client::new();
    let mut success = false;

    while retries > 0 {
        match client.get(&url).send().await {
            Ok(resp) if resp.status().is_success() => {
                success = true;
                break;
            }
            _ => {
                sleep(Duration::from_secs(1)).await;
                retries -= 1;
            }
        }
    }
    assert!(success, "Axum server did not respond with 200 OK at /");
    println!("Smoke test passed");
}
