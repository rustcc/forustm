use sapper::{SapperModule, SapperRouter, Response, Request, Result as SapperResult};
use sapper::header::ContentType;
use sapper_std::{set_cookie, JsonParams, QueryParams};
use serde_json;
use uuid::Uuid;

use super::super::{LoginUser, RegisteredUser, Redis, Postgresql};
use super::super::models::{ Articles };

pub struct Article;

impl Article {
    fn paging(req: &mut Request) -> SapperResult<Response> {
        let pg_pool = req.ext().get::<Postgresql>().unwrap().get().unwrap();

        let mut response = Response::new();
        response.headers_mut().set(ContentType::json());

        let query_params = get_query_params!(req);
        let section_id: Uuid = match t_param!(query_params, "id").clone().parse() {
            Ok(i) => i,
            Err(err) => return res_400!(format!("UUID invalid: {}", err)),
        };

        let page: i64 = match t_param_default!(query_params, "page", "1").clone().parse() {
            Ok(i) => i,
            Err(err) => return res_400!(format!("missing page param: {}", err)),
        };

        match Articles::query_articles_with_section_id_paging(&pg_pool, section_id, page) {
            Ok(arts_with_count) => {
                let res = json!({
                    "status": true,
                    "articles": arts_with_count.articles,
                    "total": arts_with_count.total,
                    "max_page": arts_with_count.max_page,
                });

                response.write_body(serde_json::to_string(&res).unwrap());
            },
            Err(err) => {
                let res = json!({
                    "status": false,
                    "error": err,
                });

                response.write_body(serde_json::to_string(&res).unwrap());
            }
        };
        Ok(response)
    }

}

impl SapperModule for Article {
    fn router(&self, router: &mut SapperRouter) -> SapperResult<()> {
        router.get("/article/paging", Article::paging);

        Ok(())
    }
}
