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
pub fn random_string(limit: usize) -> String {
    thread_rng().gen_ascii_chars().take(limit).collect()
}

/// Convert text to `sha3_256` hex
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

// define page size 
pub fn page_size() -> i64 {
    20
}
