use std::str::FromStr;
use sapper::{ SapperModule, SapperRouter, Response, Request, Result as SapperResult };
use sapper_std::{ Context, render, SessionVal, PathParams };
use super::super::{ Redis, Postgresql };
use super::super::{ Section, InsertSection };
use uuid::Uuid;

pub struct SectionDetail;

impl SectionDetail {
    fn index(req: &mut Request) -> SapperResult<Response> {
        let mut web = Context::new();
        let pg_conn = req.ext().get::<Postgresql>().unwrap().get().unwrap();

        let params = get_path_params!(req);
        let id: Uuid = t_param!(params, "id").clone().parse().unwrap();
        let res = Section::query_by_id(&pg_conn, id);
        match res {
            Ok(r) => {
                web.add("res", &r);
                res_html!("sectionDetail.html", web)
            }
            Err(e) => {
                res_500!("section not found")
            }
        }
    }
}

impl SapperModule for SectionDetail {
    fn router(&self, router: &mut SapperRouter) -> SapperResult<()> {
        router.get("/section/:id", SectionDetail::index);

        Ok(())
    }
}
