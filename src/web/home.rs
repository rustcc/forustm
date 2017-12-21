use sapper::{SapperModule, SapperRouter, Response, Request, Result as SapperResult, Error as SapperError};
use sapper_std::{Context, render, SessionVal};
use serde_json;

use super::super::{ Permissions, Redis, RUser };

pub struct Home;

impl Home {
    fn home(req: &mut Request) -> SapperResult<Response> {
        let mut web = Context::new();
        let cookie = req.ext().get::<SessionVal>().unwrap();
        let redis_pool = req.ext().get::<Redis>().unwrap();
        let info = serde_json::from_str::<RUser>(&RUser::view_with_cookie(redis_pool, cookie)).unwrap();
        web.add("user", &info);

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
