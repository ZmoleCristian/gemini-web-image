use serde_json::{json, Value};
use wreq::Client as HttpClient;

use crate::auth::Session;
use crate::endpoints;
use crate::error::Error;
use crate::parse::{self, GeneratedImage};

const WATERMARK: &str = "imagen_default.complaintflow.updated_watermark";
const FLOW_TOKEN: &str = "e15ltne15ltne15l";
const MAX_HOPS: usize = 6;

pub async fn fetch(
    http: &HttpClient,
    session: &Session,
    authuser: u32,
    image: &GeneratedImage,
) -> Result<Vec<u8>, Error> {
    let base = resolve_url(http, session, authuser, image).await?;
    let start = format!("{base}=d-I?authuser={authuser}&alr=yes");
    follow_alr(http, &start).await
}

async fn resolve_url(
    http: &HttpClient,
    session: &Session,
    authuser: u32,
    image: &GeneratedImage,
) -> Result<String, Error> {
    let inner = build_request(image);
    let inner_str = serde_json::to_string(&inner)?;
    let envelope = json!([[["c8o8Fe", inner_str, Value::Null, "generic"]]]);
    let freq = serde_json::to_string(&envelope)?;

    let url = endpoints::batchexecute(authuser);
    let source_path = format!("/u/{authuser}/app");

    let resp = http
        .post(url.as_str())
        .query(&[
            ("rpcids", "c8o8Fe"),
            ("source-path", source_path.as_str()),
            ("bl", session.build_label.as_str()),
            ("f.sid", session.session_id.as_str()),
            ("hl", "en"),
            ("pageId", "none"),
            ("_reqid", "200000"),
            ("rt", "c"),
        ])
        .header("Origin", "https://gemini.google.com")
        .header("Referer", "https://gemini.google.com/")
        .header("X-Same-Domain", "1")
        .form(&[
            ("f.req", freq.as_str()),
            ("at", session.access_token.as_str()),
        ])
        .send()
        .await?;

    let status = resp.status();
    if !status.is_success() {
        return Err(Error::BadStatus {
            status: status.as_u16(),
        });
    }

    let body = resp.text().await?;
    parse_download_url(&body)
}

fn build_request(image: &GeneratedImage) -> Value {
    let secret = json!([Value::Null, Value::Null, Value::Null, [
        Value::Null,
        Value::Null,
        Value::Null,
        Value::Null,
        Value::Null,
        image.blob
    ]]);
    let detail = json!([
        20,
        "",
        Value::Null,
        Value::Null,
        Value::Null,
        Value::Null,
        Value::Null,
        Value::Null,
        Value::Null,
        Value::Null,
        Value::Null,
        Value::Null,
        Value::Null,
        Value::Null,
        Value::Null,
        Value::Null,
        0,
        Value::Null,
        WATERMARK
    ]);
    let descriptor = json!([
        secret,
        [image.placeholder, 0],
        Value::Null,
        detail,
        Value::Null,
        Value::Null,
        Value::Null,
        Value::Null,
        Value::Null,
        FLOW_TOKEN
    ]);
    let refs = json!([
        image.response_id,
        image.candidate_id,
        image.conversation_id,
        Value::Null,
        FLOW_TOKEN
    ]);
    json!([descriptor, refs, 1, 0, 1])
}

fn parse_download_url(body: &str) -> Result<String, Error> {
    for chunk in parse::split_frames(body) {
        let frame: Value = serde_json::from_str(&chunk)?;
        let Some(parts) = frame.as_array() else {
            continue;
        };
        for part in parts {
            let Some(arr) = part.as_array() else {
                continue;
            };
            let Some(tag) = arr.first().and_then(|v| v.as_str()) else {
                continue;
            };
            if tag != "wrb.fr" {
                continue;
            }
            let Some(payload) = arr.get(2).and_then(|v| v.as_str()) else {
                continue;
            };
            let parsed: Value = serde_json::from_str(payload)?;
            let Some(url) = parsed.pointer("/0").and_then(|v| v.as_str()) else {
                continue;
            };
            return Ok(url.to_string());
        }
    }
    Err(Error::DownloadUrlAbsent)
}

async fn follow_alr(http: &HttpClient, start: &str) -> Result<Vec<u8>, Error> {
    let mut url = start.to_string();

    for _ in 0..MAX_HOPS {
        let resp = http
            .get(url.as_str())
            .header("Referer", "https://gemini.google.com/")
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            return Err(Error::BadStatus {
                status: status.as_u16(),
            });
        }

        let Some(content_type) = resp.headers().get("content-type") else {
            return Err(Error::DownloadUrlAbsent);
        };
        let is_image = content_type.as_bytes().starts_with(b"image/");

        if is_image {
            let bytes = resp.bytes().await?;
            return Ok(bytes.to_vec());
        }

        let next = resp.text().await?;
        let trimmed = next.trim();
        let Some(scheme) = trimmed.get(0..4) else {
            return Err(Error::DownloadUrlAbsent);
        };
        if scheme != "http" {
            return Err(Error::DownloadUrlAbsent);
        }
        url = trimmed.to_string();
    }

    Err(Error::DownloadUrlAbsent)
}
