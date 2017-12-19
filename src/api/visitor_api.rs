use sapper::{SapperModule, SapperRouter, Response, Request, Result as SapperResult};
use sapper::header::ContentType;
use sapper_std::{set_cookie, JsonParams, QueryParams};
use serde_json;
use uuid::Uuid;

use super::super::{LoginUser, RegisteredUser, Redis, Postgresql, RUser};
use super::super::Articles;

pub struct Visitor;

impl Visitor {
    fn login(req: &mut Request) -> SapperResult<Response> {
        let body: LoginUser = get_json_params!(req);
        let redis_pool = req.ext().get::<Redis>().unwrap();
        let pg_pool = req.ext().get::<Postgresql>().unwrap().get().unwrap();

        let mut response = Response::new();
        response.headers_mut().set(ContentType::json());

        let max_age: Option<i64> = match body.get_remember() {
            true => Some(24 * 90),
            false => None,
        };

        match body.verification(&pg_pool, redis_pool, &max_age) {
            Ok(cookies) => {
                let res = json!({
                    "status": true,
                });

                response.write_body(serde_json::to_string(&res).unwrap());

                let _ = set_cookie(&mut response,
                                   "forustm_session".to_string(),
                                   cookies,
                                   None,
                                   Some("/".to_string()),
                                   None,
                                   max_age);
            }
            Err(err) => {
                let res = json!({
                    "status": false,
                    "error": format!("{}", err)
                });

                response.write_body(serde_json::to_string(&res).unwrap());
            }
        };

        Ok(response)
    }

    fn sign_up(req: &mut Request) -> SapperResult<Response> {
        let body: RegisteredUser = get_json_params!(req);
        let pg_pool = req.ext().get::<Postgresql>().unwrap().get().unwrap();
        let redis_pool = req.ext().get::<Redis>().unwrap();

        let mut response = Response::new();
        response.headers_mut().set(ContentType::json());

        match body.register(&pg_pool, redis_pool) {
            Ok(cookies) => {
                let res = json!({
                    "status": true,
                });

                response.write_body(serde_json::to_string(&res).unwrap());

                let _ = set_cookie(&mut response,
                                   "forustm_session".to_string(),
                                   cookies,
                                   None,
                                   Some("/".to_string()),
                                   None,
                                   Some(24));
            }
            Err(err) => {
                let res = json!({
                    "status": false,
                    "error": format!("{}", err)
                });

                response.write_body(serde_json::to_string(&res).unwrap());
            }
        }
        Ok(response)
    }

    fn reset_pwd(req: &mut Request) -> SapperResult<Response> {
        #[derive(Deserialize, Serialize)]
        struct Account {
            account: String
        }
        let body: Account = get_json_params!(req);
        let pg_pool = req.ext().get::<Postgresql>().unwrap().get().unwrap();
        let res = match RUser::reset_password(&pg_pool, body.account) {
            Ok(data) => {
                json!({
                    "status": true,
                    "data": data
                })
            }
            Err(err) => {
                json!({
                    "status": false,
                    "error": err
                })
            }
        };
        res_json!(res)
    }

    fn article_paging(req: &mut Request) -> SapperResult<Response> {
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


impl SapperModule for Visitor {
    fn router(&self, router: &mut SapperRouter) -> SapperResult<()> {
        router.post("/user/login", Visitor::login);
        router.post("/user/sign_up", Visitor::sign_up);
        router.post("/user/reset_pwd", Visitor::reset_pwd);

        router.get("/article/paging", Visitor::article_paging);

        Ok(())
    }
}
