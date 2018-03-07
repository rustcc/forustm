use sapper::{Error as SapperError, Request, Response, Result as SapperResult, SapperModule,
             SapperRouter};
use sapper_std::{JsonParams, SessionVal};
use serde_json;

use super::super::{ChangePassword, DeleteArticle, DeleteComment, EditArticle, EditUser, LoginUser,
                   NewArticle, NewComment, Permissions, Postgresql, RUser, Redis, SimpleArticle,
                   UserNotify};
use super::super::get_ruser_from_session;
pub struct CommentBiz;

impl CommentBiz {
    fn new_comment(req: &mut Request) -> SapperResult<Response> {
        let body: NewComment = get_json_params!(req);
        let redis_pool = req.ext().get::<Redis>().unwrap();
        let user = get_ruser_from_session(req).unwrap();
        let pg_pool = req.ext().get::<Postgresql>().unwrap().get().unwrap();
        let article: SimpleArticle =
            SimpleArticle::query_simple_article(&pg_pool, body.article_id).unwrap();
        if let Some(reply_user_id) = body.reply_user_id {
            if user.id != reply_user_id {
                let user_reply_notify = UserNotify {
                    user_id: reply_user_id,
                    send_user_name: user.nickname.clone(),
                    article_id: article.id,
                    article_title: article.title.clone(),
                    notify_type: "reply".into(),
                };
                user_reply_notify.cache(&redis_pool);
            }
        }
        if user.id != article.author_id {
            let user_comment_notify = UserNotify {
                user_id: article.author_id,
                send_user_name: user.nickname.clone(),
                article_id: article.id,
                article_title: article.title.clone(),
                notify_type: "comment".into(),
            };
            user_comment_notify.cache(&redis_pool);
        }
        let cookie = req.ext().get::<SessionVal>().unwrap();
        res_json!(json!({
            "status": body.insert(&pg_pool, redis_pool, cookie)
        }))
    }

    fn delete_comment(req: &mut Request) -> SapperResult<Response> {
        let body: DeleteComment = get_json_params!(req);
        let cookie = req.ext().get::<SessionVal>().unwrap();
        let redis_pool = req.ext().get::<Redis>().unwrap();
        let pg_pool = req.ext().get::<Postgresql>().unwrap().get().unwrap();
        let permission = req.ext().get::<Permissions>().unwrap();
        res_json!(json!({
            "status": body.delete(&pg_pool, redis_pool, cookie, permission)
        }))
    }

}

impl SapperModule for CommentBiz {
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
        // need auth
        router.post("/comment/new", CommentBiz::new_comment);
        router.post("/comment/delete", CommentBiz::delete_comment);

        Ok(())
    }
}
