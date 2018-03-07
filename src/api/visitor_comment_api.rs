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

pub struct VisitorCommentApi;

impl VisitorCommentApi {
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
            .parse()
            .unwrap();
        let _page_size: &str = &*format!("{}", page_size());
        let limit: i64 = t_param_default!(query_params, "limit", _page_size)
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

impl SapperModule for VisitorCommentApi {
    fn router(&self, router: &mut SapperRouter) -> SapperResult<()> {
        router.get("/comment/query", VisitorCommentApi::comments_query);

        Ok(())
    }
}
