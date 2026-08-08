#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    suncode_runtime::run_http_adapter().await
}
