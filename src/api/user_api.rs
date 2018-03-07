use sapper::{Error as SapperError, Request, Response, Result as SapperResult, SapperModule,
             SapperRouter};
use sapper_std::{JsonParams, SessionVal};
use serde_json;

use super::super::{ChangePassword, DeleteArticle, DeleteComment, EditArticle, EditUser, LoginUser,
                   NewArticle, NewComment, Permissions, Postgresql, RUser, Redis, SimpleArticle,
                   UserNotify};
use super::super::get_ruser_from_session;
pub struct UserApi;

impl UserApi {
    fn view_user(req: &mut Request) -> SapperResult<Response> {
        let cookie = req.ext().get::<SessionVal>().unwrap();
        let redis_pool = req.ext().get::<Redis>().unwrap();
        let mut res = json!({
                    "status": true,
                });
        res["data"] = serde_json::from_str(&RUser::view_with_cookie(redis_pool, cookie)).unwrap();
        res_json!(res)
    }

    fn edit(req: &mut Request) -> SapperResult<Response> {
        let body: EditUser = get_json_params!(req);
        let cookie = req.ext().get::<SessionVal>().unwrap();
        let redis_pool = req.ext().get::<Redis>().unwrap();
        let pg_pool = req.ext().get::<Postgresql>().unwrap().get().unwrap();
        let res = match body.edit_user(&pg_pool, redis_pool, cookie) {
            Ok(num_edit) => json!({
                "status": true,
                "num_edit": num_edit
            }),
            Err(err) => json!({
                "status": false,
                "error": err
            }),
        };
        res_json!(res)
    }

    fn change_pwd(req: &mut Request) -> SapperResult<Response> {
        let body: ChangePassword = get_json_params!(req);
        let cookie = req.ext().get::<SessionVal>().unwrap();
        let redis_pool = req.ext().get::<Redis>().unwrap();
        let pg_pool = req.ext().get::<Postgresql>().unwrap().get().unwrap();
        let res = match body.change_password(&pg_pool, redis_pool, cookie) {
            Ok(data) => json!({
                    "status": true,
                    "data": data
                }),
            Err(err) => json!({
                    "status": false,
                    "error": err
                }),
        };
        res_json!(res)
    }

    fn sign_out(req: &mut Request) -> SapperResult<Response> {
        let cookie = req.ext().get::<SessionVal>().unwrap();
        let redis_pool = req.ext().get::<Redis>().unwrap();

        let _ = LoginUser::sign_out(redis_pool, cookie);

        res_json!(json!({
            "status": true
        }))
    }

}

impl SapperModule for UserApi {
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
        router.get("/user/sign_out", UserApi::sign_out);
        router.get("/user/view", UserApi::view_user);
        router.post("/user/change_pwd", UserApi::change_pwd);
        router.post("/user/edit", UserApi::edit);

        Ok(())
    }
}
