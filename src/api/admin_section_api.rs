use sapper::{ SapperModule, SapperRouter, Response, Request, Result as SapperResult, Error as SapperError };
use serde_json;
use sapper_std::{ JsonParams };

use super::super::{ Permissions, Postgresql, Redis, InsertSection, PubNotice };


pub struct AdminSection;

impl AdminSection {
    fn new_section(req: &mut Request) -> SapperResult<Response> {
        let body: InsertSection = get_json_params!(req);
        let pg_pool = req.ext().get::<Postgresql>().unwrap().get().unwrap();
        let res = match body.insert(&pg_pool) {
            true => json!({
                "status": true
            }),
            false => json!({
                "status": false
            }),
        };
        res_json!(res)
    }

    fn new_pub_notice(req: &mut Request) -> SapperResult<Response> {
        let body: PubNotice = get_json_params!(req);
        let redis_pool = req.ext().get::<Redis>().unwrap();
        body.insert(&redis_pool);
        res_json!(
            json!({"status": true})
        )
    }
}

impl SapperModule for AdminSection {
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
        router.post("/section/new", AdminSection::new_section);
        router.post("/pub_notice/new", AdminSection::new_pub_notice);

        Ok(())
    }
}
