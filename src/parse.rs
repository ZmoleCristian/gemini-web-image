use serde_json::Value;

use crate::error::Error;

/// An image produced by a generation request.
pub struct GeneratedImage {
    /// The image URL from the generation response (also the dedup key).
    pub url: String,
    pub(crate) blob: String,
    pub(crate) placeholder: String,
    pub(crate) candidate_id: String,
    pub(crate) response_id: String,
    pub(crate) conversation_id: String,
}

pub fn extract_images(body: &str) -> Result<Vec<GeneratedImage>, Error> {
    let mut images: Vec<GeneratedImage> = Vec::new();

    for chunk in split_frames(body) {
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
            let Some(inner_str) = arr.get(2).and_then(|v| v.as_str()) else {
                continue;
            };
            if inner_str.is_empty() {
                continue;
            }

            let inner: Value = serde_json::from_str(inner_str)?;
            let Some(conversation_id) = inner.pointer("/1/0").and_then(|v| v.as_str()) else {
                continue;
            };
            let Some(response_id) = inner.pointer("/1/1").and_then(|v| v.as_str()) else {
                continue;
            };
            let Some(candidates) = inner.get(4).and_then(|v| v.as_array()) else {
                continue;
            };

            for cand in candidates {
                let Some(candidate_id) = cand.pointer("/0").and_then(|v| v.as_str()) else {
                    continue;
                };
                for img in candidate_images(cand, candidate_id, response_id, conversation_id) {
                    if images.iter().all(|existing| existing.url != img.url) {
                        images.push(img);
                    }
                }
            }
        }
    }

    Ok(images)
}

fn candidate_images(
    cand: &Value,
    candidate_id: &str,
    response_id: &str,
    conversation_id: &str,
) -> Vec<GeneratedImage> {
    let mut out = Vec::new();

    for ptr in ["/12/7/0", "/12/0/8/0"] {
        let Some(list) = cand.pointer(ptr).and_then(|v| v.as_array()) else {
            continue;
        };
        for entry in list {
            let Some(url) = entry.pointer("/0/3/3").and_then(|v| v.as_str()) else {
                continue;
            };
            let Some(blob) = entry.pointer("/0/3/5").and_then(|v| v.as_str()) else {
                continue;
            };
            let Some(placeholder) = entry.pointer("/1/0").and_then(|v| v.as_str()) else {
                continue;
            };
            out.push(GeneratedImage {
                url: url.to_string(),
                blob: blob.to_string(),
                placeholder: placeholder.trim().to_string(),
                candidate_id: candidate_id.to_string(),
                response_id: response_id.to_string(),
                conversation_id: conversation_id.to_string(),
            });
        }
    }

    out
}

pub(crate) fn split_frames(body: &str) -> Vec<String> {
    let body = body.trim_start_matches(")]}'");
    let chars: Vec<char> = body.chars().collect();
    let total = chars.len();

    let mut frames = Vec::new();
    let mut i = 0;

    while i < total {
        while i < total && chars[i].is_whitespace() {
            i += 1;
        }

        let digit_start = i;
        while i < total && chars[i].is_ascii_digit() {
            i += 1;
        }
        if i == digit_start {
            break;
        }

        let mut length = 0usize;
        for c in &chars[digit_start..i] {
            length = length * 10 + (*c as usize - '0' as usize);
        }

        let content_start = i;
        let mut units = 0usize;
        while i < total && units < length {
            let step = if (chars[i] as u32) > 0xFFFF { 2 } else { 1 };
            units += step;
            i += 1;
        }

        let chunk: String = chars[content_start..i].iter().collect();
        let trimmed = chunk.trim();
        if !trimmed.is_empty() {
            frames.push(trimmed.to_string());
        }
    }

    frames
}
