extern crate sapper;
extern crate sapper_std;
extern crate forustm;

use sapper::{ SapperApp, SapperAppShell, Request, Response, Result as SapperResult };
use forustm::{ Redis, create_redis_pool, create_pg_pool, Postgresql, get_identity_and_web_context, Permissions,
               Visitor, User };
use forustm::{ AdminUser, AdminSection };
use std::sync::Arc;

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
        .init_global(
            Box::new(move |req: &mut Request| {
                req.ext_mut().insert::<Redis>(redis_pool.clone());
                req.ext_mut().insert::<Postgresql>(pg_pool.clone());
                Ok(())
            })
        )
        .with_shell(Box::new(ApiApp))
        .add_module(Box::new(Visitor))
        .add_module(Box::new(User))
        .add_module(Box::new(AdminUser))
        .add_module(Box::new(AdminSection))
        .static_service(false);

    println!("Start listen on {}", "0.0.0.0:8888");
    app.run_http();
}
