use std::path::{Path, PathBuf};

use gemini_web_image::{cache, load_netscape, Client};

#[tokio::main]
async fn main() {
    let prompt = std::env::args()
        .nth(1)
        .expect("usage: gen <prompt> [authuser]");
    let authuser: u32 = std::env::args()
        .nth(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);

    let cache_path = PathBuf::from(".cached_cookies.json");
    let cookies = if cache_path.exists() {
        eprintln!("loading cookies from cache");
        cache::load(&cache_path).expect("cache load")
    } else {
        eprintln!("seeding cookies from netscape export");
        load_netscape(&PathBuf::from("cookies-google-com.txt")).expect("cookies")
    };

    let client = Client::connect(&cookies, authuser).await.expect("connect");
    eprintln!("connected u/{authuser} as {}", client.email());

    cache::save(&cache_path, client.cookies()).expect("cache save");
    eprintln!("rotated + cached {} cookies", client.cookies().len());

    let images = client.generate_image(&prompt).await.expect("generate");
    eprintln!("got {} image(s)", images.len());

    for (idx, img) in images.iter().enumerate() {
        eprintln!("image {idx}: {}", img.url);
        let path = format!("out_{idx}.png");
        client
            .download_image(img, Path::new(&path))
            .await
            .expect("download");
        eprintln!("saved {path}");
    }
}
