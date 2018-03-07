#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]
#![recursion_limit = "128"]
#![deny(warnings)]

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

mod schema;

extern crate sapper;
#[macro_use] extern crate sapper_std;

use std::sync::Arc;
use sapper::{Request, Response, Result as SapperResult, SapperApp, SapperAppShell};

mod model;



mod util;
use util::{create_pg_pool, create_redis_pool, get_identity_and_web_context, Permissions,
              Postgresql, Redis};

mod api;
use api::{UserApi, VisitorApi, AdminSectionApi, AdminUserApi};





struct ApiApp;

impl SapperAppShell for ApiApp {
    fn before(&self, req: &mut Request) -> SapperResult<()> {
        sapper_std::init(req, Some("forustm_session"))?;
        let (identity, _) = get_identity_and_web_context(req);
        req.ext_mut().insert::<Permissions>(identity);
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
    app.address("0.0.0.0")
        .port(8888)
        .init_global(Box::new(move |req: &mut Request| {
            req.ext_mut().insert::<Redis>(Arc::clone(&redis_pool));
            req.ext_mut().insert::<Postgresql>(Arc::clone(&pg_pool));
            Ok(())
        }))
        .with_shell(Box::new(ApiApp))
        .add_module(Box::new(Visitor))
        .add_module(Box::new(User))
        .add_module(Box::new(AdminUser))
        .add_module(Box::new(AdminSection))
        .static_service(false);

    println!("Start listen on {}", "0.0.0.0:8888");
    app.run_http();
}
