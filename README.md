# gemini-web-image

[![crates.io](https://img.shields.io/crates/v/gemini-web-image.svg)](https://crates.io/crates/gemini-web-image)
[![license](https://img.shields.io/crate/l/gemini-web-image.svg)](#license)

Command-line / library image generation through the **web** Gemini app via cookie replay — **no official API**, no second billing surface. Drive the Gemini Pro subscription you already pay for straight from the terminal.

## ⚠️ Disclaimer

Unofficial and reverse-engineered. It replays your Google **session cookies** (which are full-account credentials) against Gemini's internal web RPCs. This is against Google's ToS and may break whenever they change the wire format. Your account, your risk. Don't point it at an account you can't afford to lose.

## Install

```sh
cargo add gemini-web-image
```

## Quickstart

```rust
use std::path::Path;
use gemini_web_image::{load_netscape, Client};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Full google.com cookie family, Netscape format (see below).
    let cookies = load_netscape(Path::new("cookies-google-com.txt"))?;

    // authuser = the /u/N index of the account you want (Pro may not be u/0).
    // connect() re-mints the short-lived 1PSIDTS before bootstrapping.
    let client = Client::connect(&cookies, 1).await?;
    println!("connected as {}", client.email());

    let images = client.generate_image("a red fox in a snowy forest, 3d render").await?;
    for (i, img) in images.iter().enumerate() {
        println!("image {i}: {}", img.url);
        client.download_image(img, Path::new(&format!("out_{i}.png"))).await?;
    }
    Ok(())
}
```

`download_bytes(&img)` returns the PNG as `Vec<u8>` if you'd rather not touch disk.

## Cookies

You need the **full `google.com` cookie family** in Netscape `cookies.txt` format — not just `__Secure-1PSID`/`1PSIDTS`. Multi-account `/u/N` targeting needs the whole set (`SID`, `SAPISID`, `HSID`, `SSID`, `APISID`, `__Secure-*PSIDCC`, `__Secure-3PSID`, …).

1. Export with any "cookies.txt" browser extension while logged into Gemini.
2. Keep only google.com rows:
   ```sh
   grep -P '^(#HttpOnly_)?[^\t]*google\.com\t' cookies.txt > cookies-google-com.txt
   ```

**Staying logged in (no re-export every run):** the durable family is what keeps you authenticated (months); `1PSIDTS` is a short-lived derivative that `connect()` re-mints, and the cookie jar self-refreshes on every response. Persist the rotated jar with `cache::save` and resume with `cache::load` — the session perpetuates itself browser-free. Re-export only if you let the chain fully rot (long downtime, or the browser desyncs it by rotating in parallel).

```rust
use gemini_web_image::cache;
// after connect():
cache::save(Path::new("/home/you/.config/gemini-web-image/cookies.json"), client.cookies())?;
// next run: let cookies = cache::load(path)?; // instead of load_netscape
```

> Keep cookie and cache files **outside any git repo**. They are full-account credentials — never commit them.

## Multi-account

The account is selected by the `/u/{N}/` path on every Gemini URL, passed as `authuser` to `connect`. Run the `whoami` example to map indices → accounts:

```sh
cargo run --example whoami -- cookies-google-com.txt
```

## How it works

Chrome TLS/JA3 impersonation (`wreq`) to dodge bot detection → GET `/u/{N}/app`, scrape the `SNlM0e` CSRF token + build label → `StreamGenerate` RPC for the prompt → `c8o8Fe` batchexecute RPC mints the download URL → follow Google's `alr=yes` redirect chain to the full-res PNG. `1PSIDTS` rotation via `accounts.google.com/RotateCookies`.

## Examples

```sh
cargo run --example gen    -- "a tiny robot watering a plant, isometric 3d" 1
cargo run --example smoke  -- cookies-google-com.txt 1   # auth bootstrap check
cargo run --example whoami -- cookies-google-com.txt     # list accounts by /u/N
```

## License

MIT
