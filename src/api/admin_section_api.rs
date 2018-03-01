use sapper::{Error as SapperError, Request, Response, Result as SapperResult, SapperModule,
             SapperRouter};
use sapper_std::JsonParams;
use serde_json;

use super::super::{InsertSection, Permissions, Postgresql, PubNotice, Redis};

pub struct AdminSection;

impl AdminSection {
    fn new_section(req: &mut Request) -> SapperResult<Response> {
        let body: InsertSection = get_json_params!(req);
        let pg_pool = req.ext().get::<Postgresql>().unwrap().get().unwrap();
        res_json!(json!({
            "status": body.insert(&pg_pool)
        }))
    }

    fn new_pub_notice(req: &mut Request) -> SapperResult<Response> {
        let body: PubNotice = get_json_params!(req);
        let redis_pool = req.ext().get::<Redis>().unwrap();
        body.insert(redis_pool);
        res_json!(json!({"status": true}))
    }
}

impl SapperModule for AdminSection {
    fn before(&self, req: &mut Request) -> SapperResult<()> {
        let permission = req.ext().get::<Permissions>().unwrap();
        match *permission {
            Some(0) => Ok(()),
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
        router.post("/section/new", AdminSection::new_section);
        router.post("/pub_notice/new", AdminSection::new_pub_notice);

        Ok(())
    }
}
