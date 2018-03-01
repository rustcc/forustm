use sapper::{Error as SapperError, Request, Response, Result as SapperResult, SapperModule,
             SapperRouter};
use sapper_std::render;

use super::super::{Permissions, WebContext};

pub struct Home;

impl Home {
    fn home(req: &mut Request) -> SapperResult<Response> {
        let web = req.ext().get::<WebContext>().unwrap().clone();
        res_html!("home.html", web)
    }
}

impl SapperModule for Home {
    fn before(&self, req: &mut Request) -> SapperResult<()> {
        let permission = req.ext().get::<Permissions>().unwrap();
        match *permission {
            Some(_) => Ok(()),
            None => Err(SapperError::TemporaryRedirect("/login".to_owned())),
        }
    }

    fn router(&self, router: &mut SapperRouter) -> SapperResult<()> {
        router.get("/home", Home::home);

        Ok(())
    }
}
