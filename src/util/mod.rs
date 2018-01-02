pub mod redis_pool;
pub mod postgresql_pool;
pub mod github_information;

pub use self::github_information::{create_https_client, get_github_nickname_and_address, get_github_primary_email,
                                   get_github_token};
pub use self::postgresql_pool::{create_pg_pool, Postgresql};
pub use self::redis_pool::{create_redis_pool, Redis, RedisPool};

use super::RUser;
use ammonia::clean;
use comrak::{markdown_to_html, ComrakOptions};
use rand::{thread_rng, Rng};
use sapper::{Client, Key, Request};
use sapper::header::ContentType;
use sapper_std::{Context, SessionVal};
use serde_json;
use serde_urlencoded;
use std::fmt::Write;
use tiny_keccak::Keccak;

/// Get random value
#[inline]
pub fn random_string(limit: usize) -> String {
    thread_rng().gen_ascii_chars().take(limit).collect()
}

/// Convert text to `sha3_256` hex
#[inline]
pub fn sha3_256_encode(s: &str) -> String {
    let mut sha3 = Keccak::new_sha3_256();
    sha3.update(s.as_ref());
    let mut res: [u8; 32] = [0; 32];
    sha3.finalize(&mut res);
    let mut hex = String::with_capacity(64);
    for byte in &res {
        write!(hex, "{:02x}", byte).expect("Can't fail on writing to string");
    }
    hex
}

/// Convert markdown to html
#[inline]
pub fn markdown_render(md: &str) -> String {
    let option = ComrakOptions {
        ext_strikethrough: true,
        ext_table: true,
        ext_tasklist: true,
        ext_superscript: true,
        ..ComrakOptions::default()
    };
    clean(&markdown_to_html(md, &option))
}

/// Get visitor status and web context
pub fn get_identity_and_web_context(req: &Request) -> (Option<i16>, Context) {
    let mut web = Context::new();
    let cookie = req.ext().get::<SessionVal>();
    let redis_pool = req.ext().get::<Redis>().unwrap();
    match cookie {
        Some(cookie) => {
            if redis_pool.exists(cookie) {
                let info = serde_json::from_str::<RUser>(&redis_pool.hget::<String>(cookie, "info")).unwrap();
                web.add("user", &info);
                (Some(info.role), web)
            } else {
                (None, web)
            }
        }
        None => (None, web),
    }
}

/// send email
#[inline]
pub fn send_reset_password_email(new_password: &str, email: &str) {
    let client = Client::new();
    let xsmtpapi = json!({
		"to": [email],
		"sub": {
			"%password%": [new_password],
			}
	});
    let body = serde_urlencoded::to_string([
        ("apiUser", "rustcc"),
        ("apiKey", "Cb2HNnzRBRGq6QLa"),
        ("templateInvokeName", "reset_password"),
        ("xsmtpapi", &xsmtpapi.to_string()),
        ("from", "admin@rust.cc"),
        ("fromName", "Admin"),
        ("subject", "重置密码"),
    ]).unwrap();
    let _ = client
        .post("http://api.sendcloud.net/apiv2/mail/sendtemplate")
        .header(ContentType::form_url_encoded())
        .body(&body)
        .send()
        .unwrap();
    println!("{} reset the password", email)
}

pub struct Permissions;

impl Key for Permissions {
    type Value = Option<i16>;
}

pub struct WebContext;

impl Key for WebContext {
    type Value = Context;
}
