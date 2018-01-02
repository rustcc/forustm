use sapper::{SapperModule, SapperRouter, Response, Request, Result as SapperResult};
use sapper::header::{ContentType, Location};
use sapper::status;
use sapper_std::{set_cookie, JsonParams, QueryParams};
use serde_json;
use uuid::Uuid;

use super::super::{LoginUser, RegisteredUser, Redis, Postgresql, RUser};
use super::super::models::{Article, CommentWithNickName};
use super::super::page_size;
use super::super::{get_github_token, get_github_nickname_and_address, create_https_client};

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

                let _ = set_cookie(
                    &mut response,
                    "forustm_session".to_string(),
                    cookies,
                    None,
                    Some("/".to_string()),
                    None,
                    max_age,
                );
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

    fn login_with_github(req: &mut Request) -> SapperResult<Response> {
        let params = get_query_params!(req);
        let code = t_param_parse!(params, "code", String);

        let redis_pool = req.ext().get::<Redis>().unwrap();
        let pg_pool = req.ext().get::<Postgresql>().unwrap().get().unwrap();
        let https_client = create_https_client();

        let token = get_github_token(&https_client, code)?;

        let mut response = Response::new();
        response.headers_mut().set(ContentType::json());

        let (nickname, github_address) = get_github_nickname_and_address(&https_client,&token)?;
        match LoginUser::login_with_github(&pg_pool, redis_pool, &https_client, github_address, nickname, token) {
            Ok(cookie) => {
                let res = json!({
                    "status": true,
                });

                response.set_status(status::Found);
                response.write_body(serde_json::to_string(&res).unwrap());
                response.headers_mut().set(Location("/home".to_owned()));

                let _ = set_cookie(
                    &mut response,
                    "forustm_session".to_string(),
                    cookie,
                    None,
                    Some("/".to_string()),
                    None,
                    Some(24),
                );
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

                let _ = set_cookie(
                    &mut response,
                    "forustm_session".to_string(),
                    cookies,
                    None,
                    Some("/".to_string()),
                    None,
                    Some(24),
                );
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
            account: String,
        }
        let body: Account = get_json_params!(req);
        if &body.account == "admin@admin.com" {
            let res = json!({
                "status": false,
                "data": "Can't change admin".to_string()
            });
            res_json!(res)
        } else {
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
    }

    fn articles_paging(req: &mut Request) -> SapperResult<Response> {
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

        match Article::query_articles_with_section_id_paging(
            &pg_pool,
            section_id,
            page,
            page_size(),
        ) {
            Ok(arts_with_count) => {
                let res = json!({
                "status": true,
                "articles": arts_with_count.articles,
                "total": arts_with_count.total,
                "max_page": arts_with_count.max_page,
            });

                response.write_body(serde_json::to_string(&res).unwrap());
            }
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

    fn article_query(req: &mut Request) -> SapperResult<Response> {
        let pg_pool = req.ext().get::<Postgresql>().unwrap().get().unwrap();

        let mut response = Response::new();
        response.headers_mut().set(ContentType::json());

        let query_params = get_query_params!(req);
        let article_id: Uuid = match t_param!(query_params, "id").clone().parse() {
            Ok(i) => i,
            Err(err) => return res_400!(format!("UUID invalid: {}", err)),
        };

        match Article::query_article_md(&pg_pool, article_id) {
            Ok(data) => {
                let res = json!({
                "status": true,
                "data": data,
            });

                response.write_body(serde_json::to_string(&res).unwrap());
            }
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

    fn blogs_paging(req: &mut Request) -> SapperResult<Response> {
        let pg_pool = req.ext().get::<Postgresql>().unwrap().get().unwrap();

        let mut response = Response::new();
        response.headers_mut().set(ContentType::json());

        let query_params = get_query_params!(req);


        let page: i64 = match t_param_default!(query_params, "page", "1").clone().parse() {
            Ok(i) => i,
            Err(err) => return res_400!(format!("missing page param: {}", err)),
        };

        match Article::query_blogs_paging(&pg_pool, 1, page, page_size()) {
            Ok(arts_with_count) => {
                let res = json!({
                    "status": true,
                    "articles": arts_with_count.articles,
                    "total": arts_with_count.total,
                    "max_page": arts_with_count.max_page,
                });

                response.write_body(serde_json::to_string(&res).unwrap());
            }
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

    fn comments_query(req: &mut Request) -> SapperResult<Response> {
        let pg_pool = req.ext().get::<Postgresql>().unwrap().get().unwrap();

        let mut response = Response::new();
        response.headers_mut().set(ContentType::json());

        let query_params = get_query_params!(req);
        let article_id: Uuid = match t_param!(query_params, "id").clone().parse() {
            Ok(i) => i,
            Err(err) => return res_400!(format!("UUID invalid: {}", err)),
        };

        let offset: i64 = t_param_default!(query_params, "offset", "0")
            .clone()
            .parse()
            .unwrap();
        let _page_size: &str = &*format!("{}", page_size());
        let limit: i64 = t_param_default!(query_params, "limit", _page_size)
            .clone()
            .parse()
            .unwrap();

        match CommentWithNickName::query(&pg_pool, limit, offset, article_id) {
            Ok(comments) => {
                let res = json!({
                    "status": true,
                    "comments": comments,
                    "loaded": comments.len()
                });

                response.write_body(serde_json::to_string(&res).unwrap());
            }
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

        router.get("/article/paging", Visitor::articles_paging);
        router.get("/article/get", Visitor::article_query);
        router.get("/blogs/paging", Visitor::blogs_paging);
        router.get("/comment/query", Visitor::comments_query);
        router.get("/login_with_github", Visitor::login_with_github);

        Ok(())
    }
}
