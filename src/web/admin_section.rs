use sapper::{SapperModule, SapperRouter, Response, Request, Result as SapperResult, Error as SapperError };
use sapper_std::{render};
use super::super::{Permissions};
use util::{get_web_context};

pub struct WebAdminSection;

impl WebAdminSection {
    fn index(req: &mut Request) -> SapperResult<Response> {
        let web = get_web_context(req);
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
