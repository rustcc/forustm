use sapper::{SapperModule, SapperRouter, Response, Request, Result as SapperResult};
use sapper_std::{render};
use super::super::{Redis, Postgresql, Article, ArticleBrief, Section, PubNotice, Permissions, WebContext};
use std::collections::HashMap;
use uuid::Uuid;

pub struct Index;


impl Index {
    fn index(req: &mut Request) -> SapperResult<Response> {
        let mut web = req.ext().get::<WebContext>().unwrap().clone();
        let redis_pool = req.ext().get::<Redis>().unwrap();
        PubNotice::get(&mut web, &redis_pool);

        let pg_conn = req.ext().get::<Postgresql>().unwrap().get().unwrap();
        let res = Article::query_articles_by_stype_paging(&pg_conn, 1, 1, 3);
        if res.is_ok(){
            web.add("blogs", &res.unwrap().articles);
        }
        let stype = 0;
        let mut sections_hash: HashMap<usize, (Uuid, String, Vec<ArticleBrief>)> = HashMap::new();
        let sections = Section::query_by_stype(&pg_conn, stype);
        if sections.is_ok(){
            web.add("sections_len", &sections.clone().unwrap().len());
            for (idx, section) in sections.unwrap().iter().enumerate() {
                let res = Article::query_articles_with_section_id_and_stype_paging(&pg_conn, section.clone().id, stype, 1, 3);
                if res.is_ok(){
                    sections_hash.insert(idx, (section.clone().id, section.clone().title, res.unwrap().articles));
                }
            }
            web.add("sections_hash", &sections_hash);
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
