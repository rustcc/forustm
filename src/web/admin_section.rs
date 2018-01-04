use super::super::{Permissions, PubNotice, Redis, WebContext};
use sapper::{Error as SapperError, Request, Response, Result as SapperResult, SapperModule, SapperRouter};
use sapper_std::render;

pub struct WebAdminSection;

impl WebAdminSection {
    fn index(req: &mut Request) -> SapperResult<Response> {
        let web = req.ext().get::<WebContext>().unwrap().clone();
        res_html!("adminSection.html", web)
    }

    fn pub_notice(req: &mut Request) -> SapperResult<Response> {
        let mut web = req.ext().get::<WebContext>().unwrap().clone();
        let redis_pool = req.ext().get::<Redis>().unwrap();
        PubNotice::get(&mut web, redis_pool);
        res_html!("adminPubNotice.html", web)
    }
}

impl SapperModule for WebAdminSection {
    fn before(&self, req: &mut Request) -> SapperResult<()> {
        let permission = req.ext().get::<Permissions>().unwrap();
        match *permission {
            Some(0) => Ok(()),
            _ => Err(SapperError::TemporaryRedirect("/login".to_owned())),
        }
    }

    fn router(&self, router: &mut SapperRouter) -> SapperResult<()> {
        router.get("/admin/section", WebAdminSection::index);
        router.get("/admin/notice", WebAdminSection::pub_notice);

        Ok(())
    }
}
