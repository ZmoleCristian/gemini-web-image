use regex::Regex;
use wreq::Client as HttpClient;

use crate::endpoints;
use crate::error::Error;

const SNLM0E: &str = r#""SNlM0e":\s*"([^"]+)""#;
const BUILD_LABEL: &str = r#""cfb2h":\s*"([^"]+)""#;
const SESSION_ID: &str = r#""FdrFJe":\s*"([^"]+)""#;
const EMAIL: &str = r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}";

pub struct Session {
    pub access_token: String,
    pub build_label: String,
    pub session_id: String,
    pub email: String,
}

pub async fn bootstrap(http: &HttpClient, authuser: u32) -> Result<Session, Error> {
    let preflight = http.get(endpoints::GOOGLE).send().await?;
    if !preflight.status().is_success() {
        return Err(Error::BadStatus {
            status: preflight.status().as_u16(),
        });
    }

    let app_url = endpoints::app(authuser);
    let app = http.get(app_url.as_str()).send().await?;
    let status = app.status();
    if !status.is_success() {
        return Err(Error::BadStatus {
            status: status.as_u16(),
        });
    }

    let html = app.text().await?;
    if html.contains("identifier-shown") || html.contains("SignIn?continue") {
        return Err(Error::NotAuthenticated);
    }

    let access_token = scrape(&html, SNLM0E)?;
    let build_label = scrape(&html, BUILD_LABEL)?;
    let session_id = scrape(&html, SESSION_ID)?;
    let email = scrape_email(&html)?;

    Ok(Session {
        access_token,
        build_label,
        session_id,
        email,
    })
}

fn scrape(html: &str, pattern: &str) -> Result<String, Error> {
    let re = Regex::new(pattern)?;
    let caps = re.captures(html).ok_or(Error::TokenAbsent)?;
    let matched = caps.get(1).ok_or(Error::TokenAbsent)?;
    Ok(matched.as_str().to_string())
}

fn scrape_email(html: &str) -> Result<String, Error> {
    let re = Regex::new(EMAIL)?;
    for found in re.find_iter(html) {
        let candidate = found.as_str();
        if !candidate.ends_with("google.com")
            && !candidate.ends_with("gstatic.com")
            && !candidate.ends_with("googleapis.com")
        {
            return Ok(candidate.to_string());
        }
    }

    Err(Error::EmailAbsent)
}
