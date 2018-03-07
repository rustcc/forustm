#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]
#![recursion_limit = "128"]
// #![deny(warnings)]

extern crate ammonia;
extern crate chrono;
extern crate comrak;
extern crate dotenv;
extern crate hyper;
extern crate hyper_native_tls;
extern crate r2d2;
extern crate r2d2_redis;
extern crate rand;
extern crate redis;
extern crate serde;
extern crate serde_urlencoded;
extern crate tiny_keccak;
extern crate uuid;

#[macro_use] extern crate serde_derive;
#[macro_use] extern crate serde_json;
#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_infer_schema;


extern crate sapper;
extern crate sapper_std;

use std::sync::Arc;
use sapper::{Request, Response, Result as SapperResult, SapperApp, SapperAppShell};

mod schema;
mod util;
mod db;
mod thirdparts;
mod model;
mod web;


pub struct Permissions;

impl Key for Permissions {
    type Value = Option<i16>;
}

pub struct WebContext;

impl Key for WebContext {
    type Value = Context;
}

struct WebApp;


/// Get visitor status and web context
pub fn get_identity_and_web_context(req: &Request) -> (Option<i16>, Context) {
    let mut web = Context::new();
    let cookie = req.ext().get::<SessionVal>();
    let redis_pool = req.ext().get::<Redis>().unwrap();
    let pg_conn = req.ext().get::<Postgresql>().unwrap().get().unwrap();
    match cookie {
        Some(cookie) => {
            if redis_pool.exists(cookie) {
                let info = serde_json::from_str::<RUser>(&redis_pool
                    .hget::<String>(cookie, "info"))
                    .unwrap();
                web.add("user", &info);
                let user_notifys = UserNotify::get_notifys(info.id, &redis_pool);
                web.add("user_notifys", &user_notifys);
                let res = Section::query_with_user_id(&pg_conn, info.id);
                if let Ok(r) = res {
                    web.add("user_blog_section", &r);
                }
                (Some(info.role), web)
            } else {
                (None, web)
            }
        }
        None => (None, web),
    }
}


impl SapperAppShell for WebApp {
    fn before(&self, req: &mut Request) -> SapperResult<()> {
        sapper_std::init(req, Some("forustm_session"))?;
        let (identity, web) = get_identity_and_web_context(req);
        req.ext_mut().insert::<Permissions>(identity);
        req.ext_mut().insert::<WebContext>(web);
        Ok(())
    }

    fn after(&self, req: &Request, res: &mut Response) -> SapperResult<()> {
        sapper_std::finish(req, res)?;
        Ok(())
    }
}

fn main() {
    let redis_pool = Arc::new(create_redis_pool(None));
    let pg_pool = create_pg_pool();
    let mut app = SapperApp::new();
    let port = 8081;
    app.address("0.0.0.0")
        .port(port)
        .init_global(Box::new(move |req: &mut Request| {
            req.ext_mut().insert::<Redis>(Arc::clone(&redis_pool));
            req.ext_mut().insert::<Postgresql>(Arc::clone(&pg_pool));
            Ok(())
        }))
        .with_shell(Box::new(WebApp))
        .add_module(Box::new(web::Index))
        .add_module(Box::new(web::WebSection))
        .add_module(Box::new(web::WebArticle))
        .add_module(Box::new(web::Home))
        .add_module(Box::new(web::WebAdminSection))
        .static_service(true);

    println!("Start listen on http://{}:{}", "0.0.0.0", port);
    app.run_http();
}
