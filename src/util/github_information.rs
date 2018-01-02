use std::io::Read;
use reqwest::Client;
use reqwest::header::Headers;
use sapper::Error as SapperError;
use serde_json;
use serde_urlencoded;

pub fn get_github_token(code: &str) -> Result<String, SapperError> {
    let params = [
        ("client_id", "3160b870124b1fcfc4cb"),
        ("client_secret", "1c970d6de12edb776bc2907689c16902c1eb909f"),
        ("code", code),
        ("accept", "json")
    ];

    Client::new()
        .post("https://github.com/login/oauth/access_token")
        .form(&params)
        .send()
        .map_err(|e| SapperError::Custom(format!("reqwest's io error: '{}'", e)))
        .and_then(|mut response| {
            let mut body = String::new();
            response.read_to_string(&mut body)
                .map_err(|e| SapperError::Custom(format!("read body error: '{}'", e)))
                .map(|_| body)
    }).and_then(|ref body| {
        #[derive(Deserialize)]
        struct Inner {
            access_token: String
        }
        serde_urlencoded::from_str::<Inner>(body)
            .map_err(|_| SapperError::Custom(String::from("No permission")))
            .map(|inner| inner.access_token)
    })
}

pub fn get_github_nickname_and_address(raw_token: &str) -> Result<(String, String), SapperError> {
    let token = serde_urlencoded::to_string([("access_token", raw_token)]).unwrap();

    let user_url = format!("https://api.github.com/user?{}", token);

    let mut header = Headers::new();
    header.append_raw("User-Agent", b"rustcc".to_vec());

    Client::new()
        .get(&user_url)
        .headers(header)
        .send()
        .map_err(|e| SapperError::Custom(format!("hyper's io error: '{}'", e)))
        .and_then(|mut response|{
            let mut body = String::new();
            response.read_to_string(&mut body)
                .map_err(|e| SapperError::Custom(format!("read body error: '{}'", e)))
                .map(|_| body)
        }).and_then(|ref body| {
            serde_json::from_str::<serde_json::Value>(body)
                .map_err(|e| SapperError::Custom(format!("read body error: '{}'", e)))
                .and_then(|inner| {
                    let nickname = match inner["name"].as_str() {
                        Some(data) => data.to_string(),
                        None => return Err(SapperError::Custom(format!("read body error")))
                    };
                    let github_address = match inner["html_url"].as_str() {
                        Some(data) => data.to_string(),
                        None => return Err(SapperError::Custom(format!("read body error")))
                    };
                    Ok((nickname, github_address))
                })
        })
}

pub fn get_github_primary_email(raw_token: &str) -> Result<String, String> {
    let token = serde_urlencoded::to_string([("access_token", raw_token)]).unwrap();

    let email_url = format!("https://api.github.com/user/emails?{}", token);
    let mut header = Headers::new();
    header.append_raw("User-Agent", b"rustcc".to_vec());

    Client::new()
        .get(&email_url)
        .headers(header)
        .send()
        .map_err(|e| format!("hyper's io error: '{}'", e))
        .and_then(|mut response|{
            let mut body = String::new();
            response.read_to_string(&mut body)
                .map_err(|e| format!("read body error: '{}'", e))
                .map(|_| body)
        }).and_then(|ref body| {
        serde_json::from_str::<Vec<serde_json::Value>>(body)
            .map_err(|e| format!("read body error: '{}'", e))
            .map(|raw_emails| {
                let primary_email = raw_emails
                    .iter()
                    .into_iter()
                    .filter(|x| x["primary"].as_bool().unwrap())
                    .map(|x| x["email"].as_str().unwrap())
                    .collect::<Vec<&str>>()
                    [0];
                primary_email.to_string()
            })
    })
}
