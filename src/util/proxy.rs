use sapper::{Request, Response, Result as SapperResult, SapperModule, SapperRouter};
use sapper_std::QueryParams;

use super::{get_github_nickname_and_address, get_github_primary_email, get_github_token};

pub struct ProxyModule;

impl ProxyModule {
    fn h_get_github_token(req: &mut Request) -> SapperResult<Response> {
        let params = get_query_params!(req);
        let code = t_param!(params, "code");

        let ret = get_github_token(code);
        if ret.is_ok() {
            res_json!(json!({
                "success": true,
                "access_token": ret.unwrap()
            }))
        } else {
            res_json!(json!({
                "success": false,
                "access_token": "".to_string()
            }))
        }
    }

    fn h_get_github_nickname_and_address(req: &mut Request) -> SapperResult<Response> {
        let params = get_query_params!(req);
        let access_token = t_param!(params, "access_token");

        let ret = get_github_nickname_and_address(access_token);
        if ret.is_ok() {
            let (nickname, github) = ret.unwrap();
            res_json!(json!({
                "success": true,
                "nickname": nickname,
                "github": github
            }))
        } else {
            res_json!(json!({
                "success": false,
                "nickname": "".to_string(),
                "github": "".to_string(),
            }))
        }
    }

    fn h_get_github_primary_email(req: &mut Request) -> SapperResult<Response> {
        let params = get_query_params!(req);
        let access_token = t_param!(params, "access_token");

        let ret = get_github_primary_email(access_token);

        if ret.is_ok() {
            let email = ret.unwrap();
            res_json!(json!({
                "success": true,
                "email": email
            }))
        } else {
            res_json!(json!({
                "success": false,
                "email": "".to_string()
            }))
        }
    }
}

impl SapperModule for ProxyModule {
    fn router(&self, router: &mut SapperRouter) -> SapperResult<()> {
        router.get("/inner/get_github_token", ProxyModule::h_get_github_token);

        router.get(
            "/inner/get_github_nickname_and_address",
            ProxyModule::h_get_github_nickname_and_address,
        );

        router.get(
            "/inner/get_github_primary_email",
            ProxyModule::h_get_github_primary_email,
        );

        Ok(())
    }
}
