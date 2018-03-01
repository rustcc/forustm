use sapper::{Error as SapperError, Request, Response, Result as SapperResult, SapperModule,
             SapperRouter};
use sapper_std::{JsonParams, QueryParams};
use serde_json;

use super::super::{ChangStatus, ChangePermission, Permissions, Postgresql, RUser};

pub struct AdminUser;

impl AdminUser {
    fn view_user_list(req: &mut Request) -> SapperResult<Response> {
        let params = get_query_params!(req);
        let limit = t_param_parse!(params, "limit", i64);
        let offset = t_param_parse!(params, "offset", i64);
        let pg_pool = req.ext().get::<Postgresql>().unwrap().get().unwrap();
        let res = match RUser::view_user_list(&pg_pool, limit, offset) {
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

    fn change_status(req: &mut Request) -> SapperResult<Response> {
        let body: ChangStatus = get_json_params!(req);
        let pg_pool = req.ext().get::<Postgresql>().unwrap().get().unwrap();

        let res = match RUser::change_status(&pg_pool, &body) {
            Ok(num_deleted) => json!({
                    "status": true,
                    "num_deleted": num_deleted
                    }),
            Err(err) => json!({
                    "status": false,
                    "error": err
                    }),
        };
        res_json!(res)
    }

    fn change_permission(req: &mut Request) -> SapperResult<Response> {
        let body: ChangePermission = get_json_params!(req);
        let pg_pool = req.ext().get::<Postgresql>().unwrap().get().unwrap();
        let res = match RUser::change_permission(&pg_pool, &body) {
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

impl SapperModule for AdminUser {
    fn before(&self, req: &mut Request) -> SapperResult<()> {
        let permission = req.ext().get::<Permissions>().unwrap();
        match *permission {
            Some(0) => Ok(()),
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
        router.get("/user/view_all", AdminUser::view_user_list);
        router.post("/user/status", AdminUser::change_status);
        router.post("/user/permission", AdminUser::change_permission);

        Ok(())
    }
}
