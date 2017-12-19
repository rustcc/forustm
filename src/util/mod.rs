pub mod redis_pool;
pub mod postgresql_pool;

pub use self::redis_pool::{create_redis_pool, RedisPool, Redis};
pub use self::postgresql_pool::{create_pg_pool, Postgresql};

use rand::{thread_rng, Rng};
use serde_json;
use tiny_keccak::Keccak;
use std::fmt::Write;
use comrak::{markdown_to_html, ComrakOptions};
use sapper::{Key, Request, Client};
use sapper::header::ContentType;
use sapper_std::SessionVal;
use super::RUser;
use serde_urlencoded;

/// Get random value
#[inline]
pub fn random_string(limit: usize) -> String {
    thread_rng().gen_ascii_chars().take(limit).collect()
}

/// Convert text to sha3_256 hex
#[inline]
pub fn sha3_256_encode(s: String) -> String {
    let mut sha3 = Keccak::new_sha3_256();
    sha3.update(s.as_ref());
    let mut res: [u8; 32] = [0; 32];
    sha3.finalize(&mut res);
    let mut hex = String::with_capacity(64);
    for byte in res.iter() {
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
    markdown_to_html(md, &option)
}

/// Get visitor status
#[inline]
pub fn get_identity(req: &Request) -> Option<i16> {
    let cookie = req.ext().get::<SessionVal>();
    let redis_pool = req.ext().get::<Redis>().unwrap();
    match cookie {
        Some(cookie) => {
            if redis_pool.exists(cookie) {
                let info =
                    serde_json::from_str::<RUser>(&redis_pool.hget::<String>(cookie, "info"))
                        .unwrap();
                Some(info.role)
            } else {
                None
            }
        }
        None => None,
    }
}

/// send email
#[inline]
pub fn send_reset_password_email(new_password: String, email: String) {
    let client = Client::new();
    let xsmtpapi = json!({
		"to": [&email],
		"sub": {
			"%password%": [&new_password],
			}
	});
    let body = serde_urlencoded::to_string(
        [
            ("apiUser", "rustcc"),
            ("apiKey","Cb2HNnzRBRGq6QLa"),
            ("templateInvokeName", "reset_password"),
            ("xsmtpapi", &xsmtpapi.to_string()),
            ("from", "admin@rust.cc"),
            ("fromName", "Admin"),
            ("subject", "重置密码")
        ]
    ).unwrap();
    let _ = client.post("http://api.sendcloud.net/apiv2/mail/sendtemplate")
        .header(ContentType::form_url_encoded())
        .body(&body)
        .send()
        .unwrap();
    println!("{} reset the password", &email)
}

pub struct Permissions;

impl Key for Permissions {
    type Value = Option<i16>;
}
