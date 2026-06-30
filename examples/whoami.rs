use std::collections::BTreeSet;
use std::fs;
use std::sync::Arc;

use regex::Regex;
use wreq::cookie::Jar;
use wreq::{Client, Url};
use wreq_util::Emulation;

#[tokio::main]
async fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "cookies.txt".to_string());

    let raw = fs::read_to_string(&path).expect("read cookie file");
    let jar = Jar::default();
    let base = Url::parse("https://gemini.google.com").expect("url");

    for line in raw.lines() {
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() < 7 {
            continue;
        }
        let domain = fields[0].trim_start_matches("#HttpOnly_");
        if !domain.contains("google.com") {
            continue;
        }
        jar.add_cookie_str(
            &format!("{}={}; Domain={domain}; Path=/; Secure", fields[5], fields[6]),
            &base,
        );
    }

    let client = Client::builder()
        .emulation(Emulation::Chrome137)
        .cookie_provider(Arc::new(jar))
        .build()
        .expect("client");

    let email_re = Regex::new(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}").expect("re");
    let snlm_re = Regex::new(r#""SNlM0e":"([^"]+)""#).expect("re2");
    let name_re = Regex::new(r#""([^"]+)",\s*"https://lh3\.googleusercontent\.com[^"]*"#).expect("re3");

    for n in 0..6u32 {
        let url = format!("https://gemini.google.com/u/{n}/app");
        let resp = client.get(&url).send().await.expect("send");
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();

        let active = snlm_re.is_match(&body);
        let emails: BTreeSet<&str> = email_re
            .find_iter(&body)
            .map(|m| m.as_str())
            .filter(|e| {
                !e.ends_with("google.com")
                    && !e.ends_with("gstatic.com")
                    && !e.ends_with("googleapis.com")
                    && !e.ends_with("gmail.com.png")
            })
            .collect();
        let name = name_re
            .captures(&body)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str())
            .unwrap_or("?");

        eprintln!(
            "u/{n}: status={status} active={active} name={name:?} emails={:?}",
            emails.iter().take(6).collect::<Vec<_>>()
        );
    }
}
