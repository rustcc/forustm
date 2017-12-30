use hyper::Client;
use hyper::net::HttpsConnector;
use hyper_native_tls::NativeTlsClient;
use std::io::Read;
use hyper::header::ContentType;
use hyper::header::Headers;
use std::collections::HashMap;
use serde_urlencoded;
use serde_json;
use sapper::Error as SapperError;

pub fn get_github_token(code: String) -> Result<String, SapperError> {
    let ssl = NativeTlsClient::new().unwrap();
    let connector = HttpsConnector::new(ssl);
    let client = Client::with_connector(connector);
    let body = serde_urlencoded::to_string(
        [
            ("client_id", "3160b870124b1fcfc4cb"),
            ("client_secret", "1c970d6de12edb776bc2907689c16902c1eb909f"),
            ("code", &code),
            ("accept", "json"),
        ],
    ).unwrap();

    let mut res = client
        .post("https://github.com/login/oauth/access_token")
        .header(ContentType::form_url_encoded())
        .body(&body)
        .send()
        .unwrap();

    let mut text = String::new();
    res.read_to_string(&mut text).unwrap();

    let res_decode = serde_urlencoded::from_str::<HashMap<String, String>>(&text).unwrap();

    if res_decode.contains_key("access_token") {
        Ok(res_decode.get("access_token").unwrap().to_string())
    } else {
        Err(SapperError::Custom("No permission".to_string()))
    }
}

pub fn get_github_nickname_and_address(raw_token: &str) -> (String, String) {
    let ssl = NativeTlsClient::new().unwrap();
    let connector = HttpsConnector::new(ssl);
    let client = Client::with_connector(connector);

    let token = serde_urlencoded::to_string([("access_token", raw_token)]).unwrap();

    let user_url = format!("https://api.github.com/user?{}", token);

    let mut header = Headers::new();
    header.append_raw("User-Agent", b"rustcc".to_vec());

    let mut raw_user_res = client
        .get(&user_url)
        .headers(header.clone())
        .send()
        .unwrap();
    let mut text = String::new();
    raw_user_res.read_to_string(&mut text).unwrap();

    let raw_user: serde_json::Value = serde_json::from_str(&text).unwrap();
    let nickname = raw_user["name"].as_str().unwrap().to_string();
    let github_address = raw_user["html_url"].as_str().unwrap().to_string();

    (nickname, github_address)
}

pub fn get_github_primary_email(raw_token: &str) -> String {
    let ssl = NativeTlsClient::new().unwrap();
    let connector = HttpsConnector::new(ssl);
    let client = Client::with_connector(connector);

    let token = serde_urlencoded::to_string([("access_token", raw_token)]).unwrap();

    let email_url = format!("https://api.github.com/user/emails?{}", token);
    let mut header = Headers::new();
    header.append_raw("User-Agent", b"rustcc".to_vec());

    let mut raw_emails_res = client.get(&email_url).headers(header).send().unwrap();
    let mut text = String::new();
    raw_emails_res.read_to_string(&mut text).unwrap();

    let raw_emails: Vec<serde_json::Value> = serde_json::from_str(&text).unwrap();
    let primary_email = raw_emails
        .iter()
        .into_iter()
        .filter(|x| x["primary"].as_bool().unwrap())
        .map(|x| x["email"].as_str().unwrap())
        .collect::<Vec<&str>>()
        [0];

    primary_email.to_string()
}
