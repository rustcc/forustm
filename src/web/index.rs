use sapper::{SapperModule, SapperRouter, Response, Request, Result as SapperResult};
use sapper_std::{Context, render, SessionVal, PathParams};
use uuid::Uuid;

pub struct Index;

impl Index {
    fn index(req: &mut Request) -> SapperResult<Response> {
        let web = Context::new();
        res_html!("index.html", web)
    }

    fn login(_req: &mut Request) -> SapperResult<Response> {
        let web = Context::new();
        res_html!("login.html", web)
    }
}

impl SapperModule for Index {
    fn router(&self, router: &mut SapperRouter) -> SapperResult<()> {
        router.get("/", Index::index);

        router.get("/login", Index::login);

        Ok(())
    }
}
