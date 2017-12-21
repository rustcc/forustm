use sapper::{SapperModule, SapperRouter, Response, Request, Result as SapperResult};
use sapper_std::{Context, render, SessionVal};
use super::super::Redis;
use super::super::{RUser, Permissions};
use serde_json;

pub struct Index;

impl Index {
    fn index(req: &mut Request) -> SapperResult<Response> {
        let mut web = Context::new();
        let redis_pool = req.ext().get::<Redis>().unwrap();
        let cookie = req.ext().get::<SessionVal>();
        if cookie.is_some() {
            let user: RUser = serde_json::from_str(&RUser::view_with_cookie(redis_pool, cookie.unwrap())).unwrap();
            web.add("user", &user);
        }
        res_html!("index.html", web)
    }

    fn login(req: &mut Request) -> SapperResult<Response> {
        let permission = req.ext().get::<Permissions>().unwrap().to_owned();
        let web = Context::new();
        match permission {
            Some(_) => {
                res_redirect!("/home")
            },
            None => {
                res_html!("login.html", web)
            }
        }
    }
}

impl SapperModule for Index {

    fn router(&self, router: &mut SapperRouter) -> SapperResult<()> {
        router.get("/", Index::index);

        router.get("/login", Index::login);

        Ok(())
    }
}
