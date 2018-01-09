use std::io::Read;
use sapper::Error as SapperError;
use serde_json;
use serde_urlencoded;
use hyper::Client;

use std::thread;
use std::sync::mpsc;


pub fn inner_get_github_token(code: &str) -> Result<String, SapperError> {
    let _code = code.to_owned();
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let client = Client::new();
        
        let params = serde_urlencoded::to_string([("code", &_code[..])]).unwrap();

        let url = format!("http://127.0.0.1:7777/inner/get_github_token?{}", params);

        let ret = client.get(&url)
            .send()
            .map_err(|e| SapperError::Custom(format!("hyper's io error: '{}'", e)))
            .and_then(|mut response| {
                let mut body = String::new();
                response.read_to_string(&mut body)
                    .map_err(|e| SapperError::Custom(format!("read body error: '{}'", e)))
                    .map(|_| body)
            }).and_then(|ref body| {
                #[derive(Deserialize)]
                struct RetVal {
                    success: bool,
                    access_token: String
                }
                serde_json::from_str::<RetVal>(body)
                    .map_err(|_| SapperError::Custom(String::from("parsing return json error")))
                    .map(|inner| inner)
            });
        
        let tosend = match ret {
            Ok(jsonret) => {
                if jsonret.success == true { 
                    Ok(jsonret.access_token)
                }
                else {
                    Err(SapperError::Custom(String::from("return json is false")))
                }
            },
            Err(err) => {
                Err(err)
            }
        };

        tx.send(tosend).unwrap();

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

                #[derive(Deserialize)]
                struct RetVal {
                    success: bool,
                    nickname: String,
                    github: String
                }
                serde_json::from_str::<RetVal>(body)
                    .map_err(|_| SapperError::Custom(String::from("return json parsing error")))
                    .map(|inner| {
                        inner
                    })
            });
        
        let tosend = match ret {
            Ok(jsonret) => {
                if jsonret.success == true { 
                    Ok((jsonret.nickname, jsonret.github))
                }
                else {
                    Err(SapperError::Custom(String::from("return json is false")))
                }
            },
            Err(err) => {
                Err(err)
            }
        };

        tx.send(tosend).unwrap();
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

        let email_url = format!("http://127.0.0.1:7777/inner/get_github_primary_email?{}", token);

        let ret = client.get(&email_url)
            .send()
            .map_err(|e| SapperError::Custom(format!("hyper's io error: '{}'", e)))
            .and_then(|mut response|{
                let mut body = String::new();
                response.read_to_string(&mut body)
                    .map_err(|e| SapperError::Custom(format!("read body error: '{}'", e)))
                    .map(|_| body)
            }).and_then(|ref body| {
                #[derive(Deserialize)]
                struct RetVal {
                    success: bool,
                    email: String
                }
                serde_json::from_str::<RetVal>(body)
                    .map_err(|_| SapperError::Custom(String::from("return json parsing error")))
                    .map(|inner| {
                        inner
                    })
            });

        let tosend = match ret {
            Ok(jsonret) => {
                if jsonret.success == true { 
                    Ok(jsonret.email)
                }
                else {
                    Err(String::from("return json is false"))
                }
            },
            Err(_) => {
                Err(String::from("parsing error"))
            }
        };

        tx.send(tosend).unwrap();
    });

    let received: Result<String, String> = rx.recv().unwrap();
    println!("Got: {:?}", received);

    received
}
