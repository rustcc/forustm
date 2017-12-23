use sapper::{SapperModule, SapperRouter, Response, Request, Result as SapperResult};
use sapper_std::{render};
use super::super::{Redis, Postgresql, Article, PubNotice, Permissions, WebContext};

pub struct Index;

impl Index {
    fn index(req: &mut Request) -> SapperResult<Response> {
        let mut web = req.ext().get::<WebContext>().unwrap().clone();
        let redis_pool = req.ext().get::<Redis>().unwrap();
        PubNotice::get(&mut web, &redis_pool);

        let pg_conn = req.ext().get::<Postgresql>().unwrap().get().unwrap();
        let res = Article::query_articles_by_stype_paging(&pg_conn, 1, 1, 3);
        if res.is_ok(){
            web.add("articles", &res.unwrap().articles);
        }
        res_html!("index.html", web)
    }

    fn login(req: &mut Request) -> SapperResult<Response> {
        let permission = req.ext().get::<Permissions>().unwrap().to_owned();
        let web = req.ext().get::<WebContext>().unwrap().clone();
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
