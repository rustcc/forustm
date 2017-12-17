extern crate sapper;
extern crate sapper_std;
extern crate forustm;

use std::sync::Arc;
use sapper::{ SapperApp, SapperAppShell, Request, Response, Result as SapperResult };
use forustm::{ Redis, create_redis_pool, create_pg_pool, Postgresql, get_identity, Permissions };
use forustm::{Index};

struct WebApp;

impl SapperAppShell for WebApp {
    fn before(&self, req: &mut Request) -> SapperResult<()> {
        sapper_std::init(req, Some("forustm_session"))?;
        let identity = get_identity(req);
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
    let port = 8081;
    app.address("127.0.0.1")
        .port(port)
        .init_global(
            Box::new(move |req: &mut Request| {
                req.ext_mut().insert::<Redis>(redis_pool.clone());
                req.ext_mut().insert::<Postgresql>(pg_pool.clone());
                Ok(())
            })
        )
        .with_shell(Box::new(WebApp))
        .add_module(Box::new(Index))
        .static_service(true);

    println!("Start listen on http://{}:{}", "127.0.0.1", port);
    app.run_http();
}
