use std::io::Read;
use sapper::Error as SapperError;
use serde_json;
use serde_urlencoded;
use hyper::Client;
use hyper::header::ContentType;

use std::thread;
use std::sync::mpsc;


pub fn inner_get_github_token(code: &str) -> Result<String, SapperError> {
    let _code = code.to_owned();
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let client = Client::new();
        
        let params = serde_urlencoded::to_string(
            [
            ("code", &_code[..]),
            ],
            ).unwrap();

        let ret = client.post("http://127.0.0.1:7777/inner/get_github_token")
            .header(ContentType::form_url_encoded())
            .body(&params)
            .send()
            .map_err(|e| SapperError::Custom(format!("hyper's io error: '{}'", e)))
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
            });

        tx.send(ret).unwrap();
    });

    let received: Result<String, SapperError> = rx.recv().unwrap();
    println!("Got: {:?}", received);

    received
}

pub fn inner_get_github_nickname_and_address(raw_token: &str) -> Result<(String, String), SapperError> {
    let token = serde_urlencoded::to_string([("access_token", raw_token)]).unwrap();

    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let client = Client::new();

        let user_url = format!("http://127.0.0.1:7777/inner/get_github_nickname_and_address?{}", token);

        let ret = client.get(&user_url)
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
            });

        tx.send(ret).unwrap();
    });

    let received: Result<(String, String), SapperError> = rx.recv().unwrap();
    println!("Got: {:?}", received);

    received
}

pub fn inner_get_github_primary_email(raw_token: &str) -> Result<String, String> {
    let token = serde_urlencoded::to_string([("access_token", raw_token)]).unwrap();

    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let client = Client::new();

        let email_url = format!("http://127.0.0.1/inner/get_github_primary_email?{}", token);

        let ret = client.get(&email_url)
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
            });

        tx.send(ret).unwrap();
    });

    let received: Result<String, String> = rx.recv().unwrap();
    println!("Got: {:?}", received);

    received
}
