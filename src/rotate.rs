use wreq::cookie::Cookie as RawCookie;
use wreq::Client as HttpClient;

use crate::cookies::Cookie;
use crate::endpoints;
use crate::error::Error;

const ROTATE_BODY: &str = "[000,\"-0000000000000000000\"]";

pub async fn rotate(http: &HttpClient, cookies: &mut Vec<Cookie>) -> Result<(), Error> {
    let resp = http
        .post(endpoints::ROTATE)
        .header("Content-Type", "application/json")
        .header("Origin", "https://accounts.google.com")
        .body(ROTATE_BODY.to_string())
        .send()
        .await?;

    let status = resp.status();
    if status.as_u16() == 401 {
        return Err(Error::NotAuthenticated);
    }
    if !status.is_success() {
        return Err(Error::BadStatus {
            status: status.as_u16(),
        });
    }

    for value in resp.headers().get_all("set-cookie") {
        let parsed = RawCookie::parse(value.as_bytes())?;
        apply(cookies, parsed.name(), parsed.value());
    }

    Ok(())
}

fn apply(cookies: &mut Vec<Cookie>, name: &str, value: &str) {
    for existing in cookies.iter_mut() {
        if existing.name == name {
            existing.value = value.to_string();
            return;
        }
    }
    cookies.push(Cookie {
        domain: ".google.com".to_string(),
        name: name.to_string(),
        value: value.to_string(),
    });
}
