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

mod schema;

extern crate sapper;
extern crate sapper_std;

use std::sync::Arc;
use sapper::{Request, Response, Result as SapperResult, SapperApp, SapperAppShell};

mod model;
use model::*;

mod util;
use util::{create_pg_pool, create_redis_pool, Postgresql, Redis, get_identity_and_web_context, Permissions, WebContext};

mod web;
use web::*;

struct WebApp;

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
        .add_module(Box::new(Index))
        .add_module(Box::new(WebSection))
        .add_module(Box::new(WebArticle))
        .add_module(Box::new(Home))
        .add_module(Box::new(WebAdminSection))
        .static_service(true);

    println!("Start listen on http://{}:{}", "0.0.0.0", port);
    app.run_http();
}
