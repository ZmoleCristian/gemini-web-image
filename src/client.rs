use std::fs;
use std::path::Path;
use std::sync::Arc;

use wreq::cookie::Jar;
use wreq::redirect::Policy;
use wreq::{Client as HttpClient, Url};
use wreq_util::Emulation;

use crate::auth::{self, Session};
use crate::cookies::Cookie;
use crate::download;
use crate::error::Error;
use crate::generate;
use crate::parse::{self, GeneratedImage};
use crate::rotate;

const ORIGIN: &str = "https://gemini.google.com";

pub struct Client {
    http: HttpClient,
    session: Session,
    authuser: u32,
    cookies: Vec<Cookie>,
}

impl Client {
    pub async fn connect(cookies: &[Cookie], authuser: u32) -> Result<Client, Error> {
        let mut cookies = cookies.to_vec();
        let jar = Jar::default();
        let origin = Url::parse(ORIGIN)?;

        for cookie in &cookies {
            jar.add_cookie_str(
                &format!(
                    "{}={}; Domain={}; Path=/; Secure",
                    cookie.name, cookie.value, cookie.domain
                ),
                &origin,
            );
        }

        let http = HttpClient::builder()
            .emulation(Emulation::Chrome137)
            .cookie_provider(Arc::new(jar))
            .redirect(Policy::limited(10))
            .build()?;

        rotate::rotate(&http, &mut cookies).await?;
        let session = auth::bootstrap(&http, authuser).await?;

        Ok(Client {
            http,
            session,
            authuser,
            cookies,
        })
    }

    pub async fn refresh(&mut self) -> Result<(), Error> {
        rotate::rotate(&self.http, &mut self.cookies).await
    }

    pub fn cookies(&self) -> &[Cookie] {
        &self.cookies
    }

    pub async fn generate_image(&self, prompt: &str) -> Result<Vec<GeneratedImage>, Error> {
        let body = generate::generate(&self.http, &self.session, self.authuser, prompt).await?;
        parse::extract_images(&body)
    }

    pub async fn download_bytes(&self, image: &GeneratedImage) -> Result<Vec<u8>, Error> {
        download::fetch(&self.http, &self.session, self.authuser, image).await
    }

    pub async fn download_image(&self, image: &GeneratedImage, path: &Path) -> Result<(), Error> {
        let bytes = self.download_bytes(image).await?;
        fs::write(path, &bytes)?;
        Ok(())
    }

    pub fn email(&self) -> &str {
        &self.session.email
    }
}
