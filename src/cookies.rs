use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::Error;

#[derive(Clone, Serialize, Deserialize)]
pub struct Cookie {
    pub domain: String,
    pub name: String,
    pub value: String,
}

pub fn load_netscape(path: &Path) -> Result<Vec<Cookie>, Error> {
    let raw = fs::read_to_string(path)?;

    let mut cookies = Vec::new();
    let mut has_psid = false;

    for line in raw.lines() {
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() < 7 {
            continue;
        }

        let domain = fields[0].trim_start_matches("#HttpOnly_");
        if !domain.contains("google.com") {
            continue;
        }

        let name = fields[5];
        if name == "__Secure-1PSID" {
            has_psid = true;
        }

        cookies.push(Cookie {
            domain: domain.to_string(),
            name: name.to_string(),
            value: fields[6].to_string(),
        });
    }

    if !has_psid {
        return Err(Error::CookieMissing);
    }

    Ok(cookies)
}
