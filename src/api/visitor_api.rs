use sapper::{Request, Response, Result as SapperResult, SapperModule, SapperRouter};
use sapper::header::{ContentType, Location};
use sapper::status;
use sapper_std::{set_cookie, JsonParams, QueryParams};
use serde_json;
use uuid::Uuid;

use super::super::{LoginUser, NewArticleStats, Postgresql, RUser, Redis, RegisteredUser,
                   UserNotify};
use super::super::{inner_get_github_nickname_and_address, inner_get_github_token};
use super::super::models::{Article, CommentWithNickName};
use super::super::page_size;
use super::super::{get_real_ip_from_req, get_ruser_from_session, get_user_agent_from_req};

pub struct Visitor;

impl Visitor {
    fn login(req: &mut Request) -> SapperResult<Response> {
        let body: LoginUser = get_json_params!(req);
        let redis_pool = req.ext().get::<Redis>().unwrap();
        let pg_pool = req.ext().get::<Postgresql>().unwrap().get().unwrap();

        let mut response = Response::new();
        response.headers_mut().set(ContentType::json());

        let max_age = if body.get_remember() {
            Some(24 * 90)
        } else {
            None
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

        let token = inner_get_github_token(&code)?;

        let mut response = Response::new();
        response.headers_mut().set(ContentType::json());

        let (nickname, github_address) = inner_get_github_nickname_and_address(&token)?;
        match LoginUser::login_with_github(&pg_pool, redis_pool, github_address, nickname, &token) {
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

    fn send_reset_pwd_email(req: &mut Request) -> SapperResult<Response> {
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
            let redis_pool = req.ext().get::<Redis>().unwrap();
            let res = match RUser::send_reset_pwd_email(&pg_pool, redis_pool, body.account) {
                Ok(_) => json!({
                    "status": true
                }),
                Err(err) => json!({
                    "status": false,
                    "error": err
                }),
            };
            res_json!(res)
        }
    }

    fn reset_pwd(req: &mut Request) -> SapperResult<Response> {
        #[derive(Deserialize, Serialize)]
        struct Massage {
            password: String,
            cookie: String,
        }
        let mut response = Response::new();
        response.headers_mut().set(ContentType::json());

        let body: Massage = get_json_params!(req);
        let redis_pool = req.ext().get::<Redis>().unwrap();
        let pg_pool = req.ext().get::<Postgresql>().unwrap().get().unwrap();

        match RUser::reset_pwd(&pg_pool, redis_pool, body.password, body.cookie) {
            Ok(cookie) => {
                let res = json!({
                    "status": true,
                });

                response.write_body(serde_json::to_string(&res).unwrap());

                let _ = set_cookie(
                    &mut response,
                    "forustm_session".to_string(),
                    cookie,
                    None,
                    Some("/".to_string()),
                    None,
                    None,
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

}

impl SapperModule for Visitor {
    fn router(&self, router: &mut SapperRouter) -> SapperResult<()> {
        router.post("/user/login", Visitor::login);
        router.post("/user/sign_up", Visitor::sign_up);
        router.post("/user/send_reset_pwd_email", Visitor::send_reset_pwd_email);
        router.post("/user/reset_pwd", Visitor::reset_pwd);
        router.get("/login_with_github", Visitor::login_with_github);

        Ok(())
    }
}
