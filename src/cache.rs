use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use crate::cookies::Cookie;
use crate::error::Error;

pub fn save(path: &Path, cookies: &[Cookie]) -> Result<(), Error> {
    let json = serde_json::to_string(cookies)?;
    fs::write(path, json)?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    Ok(())
}

pub fn load(path: &Path) -> Result<Vec<Cookie>, Error> {
    let raw = fs::read_to_string(path)?;
    let cookies = serde_json::from_str(&raw)?;
    Ok(cookies)
}
