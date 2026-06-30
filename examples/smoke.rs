use std::path::PathBuf;

use gemini_web_image::{load_netscape, Client};

#[tokio::main]
async fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "cookies.txt".to_string());
    let authuser: u32 = std::env::args()
        .nth(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let cookies = load_netscape(&PathBuf::from(&path)).expect("load cookies");
    eprintln!("loaded {} google cookies", cookies.len());

    let client = Client::connect(&cookies, authuser)
        .await
        .expect("bootstrap connect");
    eprintln!("connected u/{authuser} as {}", client.email());
}
