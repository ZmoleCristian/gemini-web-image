use thiserror::Error;

/// Everything that can go wrong talking to web Gemini.
#[derive(Debug, Error)]
pub enum Error {
    #[error("http transport failed: {0}")]
    Http(#[from] wreq::Error),
    #[error("regex compilation failed: {0}")]
    Regex(#[from] regex::Error),
    #[error("base url parse failed: {0}")]
    Url(#[from] url::ParseError),
    #[error("gemini endpoint returned status {status}")]
    BadStatus { status: u16 },
    #[error("login redirect detected; cookies invalid or expired")]
    NotAuthenticated,
    #[error("required session token absent from app html")]
    TokenAbsent,
    #[error("cookie file io failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("__Secure-1PSID not found in cookie file")]
    CookieMissing,
    #[error("account email absent from app html")]
    EmailAbsent,
    #[error("json parse failed: {0}")]
    Json(#[from] serde_json::Error),
    #[error("download url absent from batchexecute response")]
    DownloadUrlAbsent,
}
