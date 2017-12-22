use sapper::{SapperModule, SapperRouter, Response, Request, Result as SapperResult, Error as SapperError};
use sapper_std::{render};

use util::{get_web_context};

use super::super::{ Permissions };

pub struct Home;

impl Home {
    fn home(req: &mut Request) -> SapperResult<Response> {
        let web = get_web_context(req);
        res_html!("home.html", web)
    }
}

impl SapperModule for Home {
    fn before(&self, req: &mut Request) -> SapperResult<()> {
        let permission = req.ext().get::<Permissions>().unwrap();
        match permission {
            &Some(_) => Ok(()),
            &None => {
                let res = json!({
                    "status": false,
                    "error": String::from("Verification error")
                });
                Err(SapperError::CustomJson(res.to_string()))
            }
        }
    }

    fn router(&self, router: &mut SapperRouter) -> SapperResult<()> {
        router.get("/home", Home::home);

        Ok(())
    }
}
