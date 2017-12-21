use sapper::{SapperModule, SapperRouter, Response, Request, Result as SapperResult, Error as SapperError };
use sapper_std::{Context, render, SessionVal};
use super::super::Redis;
use super::super::{RUser, Permissions};
use serde_json;

pub struct WebAdminSection;

impl WebAdminSection {
    fn index(req: &mut Request) -> SapperResult<Response> {
        let mut web = Context::new();

        let redis_pool = req.ext().get::<Redis>().unwrap();
        let cookie = req.ext().get::<SessionVal>().unwrap();
        let user: RUser = serde_json::from_str(&RUser::view_with_cookie(redis_pool, cookie)).unwrap();

        web.add("user", &user);
        res_html!("adminSection.html", web)
    }
}

impl SapperModule for WebAdminSection {
    fn before(&self, req: &mut Request) -> SapperResult<()> {
        let permission = req.ext().get::<Permissions>().unwrap();
        match permission {
            &Some(0) => Ok(()),
            _ => {
                let res = json!({
                    "status": false,
                    "error": String::from("Verification error")
                });
                Err(SapperError::CustomJson(res.to_string()))
            }
        }
    }

    fn router(&self, router: &mut SapperRouter) -> SapperResult<()> {
        router.get("/admin/section", WebAdminSection::index);

        Ok(())
    }
}

