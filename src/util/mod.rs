pub mod redis_pool;
pub mod postgresql_pool;
pub mod inner_http;
pub mod github_information;

pub use self::inner_http::{inner_get_github_nickname_and_address, inner_get_github_primary_email,
                           inner_get_github_token};
pub use self::github_information::{get_github_nickname_and_address, get_github_primary_email, get_github_token};
pub use self::postgresql_pool::{create_pg_pool, Postgresql};
pub use self::redis_pool::{create_redis_pool, Redis, RedisPool};

use super::RUser;
use super::UserNotify;
use super::Section;
use ammonia::clean;
use comrak::{markdown_to_html, ComrakOptions};
use rand::{thread_rng, Rng};
use sapper::{Client, Key, Request};
use sapper::header::ContentType;
use sapper::header::UserAgent;
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
    let pg_conn = req.ext().get::<Postgresql>().unwrap().get().unwrap();
    match cookie {
        Some(cookie) => {
            if redis_pool.exists(cookie) {
                let info = serde_json::from_str::<RUser>(&redis_pool.hget::<String>(cookie, "info")).unwrap();
                web.add("user", &info);
                let user_notifys = UserNotify::get_notifys(info.id, &redis_pool);
                web.add("user_notifys", &user_notifys);
                let res = Section::query_with_user_id(&pg_conn, info.id);
                if let Ok(r) = res {
                    web.add("user_blog_section", &r);
                }
                (Some(info.role), web)
            } else {
                (None, web)
            }
        }
        None => (None, web),
    }
}

/// send email
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

/// get user if login or none
pub fn get_ruser_from_session(req: &Request) -> Option<RUser> {
    let redis_pool = req.ext().get::<Redis>().unwrap();
    match req.ext().get::<SessionVal>() {
        Some(cookie) => {
            if redis_pool.exists(cookie) {
                let redis_pool = req.ext().get::<Redis>().unwrap();
                let user: RUser = serde_json::from_str(&RUser::view_with_cookie(redis_pool, cookie)).unwrap();
                Some(user)
            } else {
                None
            }
        }
        None => None,
    }
}

/// get request's real ip when request proxyed by nginx or normal ip
pub fn get_real_ip_from_req(req: &Request) -> Option<String> {
    match req.headers().get_raw("X-Real-IP") {
        Some(fip) => String::from_utf8((*fip)[0].clone()).ok(),
        None => serde_json::to_string(&req.remote_addr().ip())
            .ok()
            .map(|s| String::from(&s[1..s.len() - 1])),
    }
}

/// get request's user-agent
pub fn get_user_agent_from_req(req: &Request) -> Option<String> {
    req.headers()
        .get::<UserAgent>()
        .map(|user_agent| String::from(user_agent.trim()))
}

pub struct Permissions;

impl Key for Permissions {
    type Value = Option<i16>;
}

pub struct WebContext;

impl Key for WebContext {
    type Value = Context;
}
