use sapper::{Error as SapperError, Request, Response, Result as SapperResult, SapperModule,
             SapperRouter};
use sapper_std::{JsonParams, SessionVal};
use serde_json;

use super::super::{ChangePassword, DeleteArticle, DeleteComment, EditArticle, EditUser, LoginUser,
                   NewArticle, NewComment, Permissions, Postgresql, RUser, Redis, SimpleArticle,
                   UserNotify};
use super::super::get_ruser_from_session;

pub struct ArticleApi;

impl ArticleApi {

    fn new_article(req: &mut Request) -> SapperResult<Response> {
        let body: NewArticle = get_json_params!(req);
        let pg_pool = req.ext().get::<Postgresql>().unwrap().get().unwrap();
        let cookie = req.ext().get::<SessionVal>().unwrap();
        let redis_pool = req.ext().get::<Redis>().unwrap();
        match body.insert(&pg_pool, redis_pool, cookie) {
            Ok(_) => res_json!(json!({"status": true})),

            Err(t) => res_json!(json!({"status": false, "error": t})),
        }
    }

    fn delete_article(req: &mut Request) -> SapperResult<Response> {
        let body: DeleteArticle = get_json_params!(req);
        let pg_pool = req.ext().get::<Postgresql>().unwrap().get().unwrap();
        let redis_pool = req.ext().get::<Redis>().unwrap();
        let permission = req.ext().get::<Permissions>().unwrap();
        let cookie = req.ext().get::<SessionVal>().unwrap();
        res_json!(json!({
            "status": body.delete(&pg_pool, redis_pool, cookie, permission)
        }))
    }

    fn edit_article(req: &mut Request) -> SapperResult<Response> {
        let body: EditArticle = get_json_params!(req);
        let redis_pool = req.ext().get::<Redis>().unwrap();
        let pg_pool = req.ext().get::<Postgresql>().unwrap().get().unwrap();
        let cookie = req.ext().get::<SessionVal>().unwrap();

        let res = match body.edit_article(&pg_pool, redis_pool, cookie) {
            Ok(num_update) => json!({
                    "status": true,
                    "num_update": num_update
                }),
            Err(err) => json!({
                    "status": false,
                    "error": format!("{}", err)
                }),
        };
        res_json!(res)
    }
}

impl SapperModule for ArticleApi {
    fn before(&self, req: &mut Request) -> SapperResult<()> {
        let permission = req.ext().get::<Permissions>().unwrap();
        match *permission {
            Some(_) => Ok(()),
            None => {
                let res = json!({
                    "status": false,
                    "error": String::from("Verification error")
                });
                Err(SapperError::CustomJson(res.to_string()))
            }
        }
    }

    fn router(&self, router: &mut SapperRouter) -> SapperResult<()> {
        router.post("/article/new", ArticleApi::new_article);
        router.post("/article/delete", ArticleApi::delete_article);
        router.post("/article/edit", ArticleApi::edit_article);

        Ok(())
    }
}
