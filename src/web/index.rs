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
        
        // query blogs
        let res = Article::query_articles_by_stype_paging(&pg_conn, 1, 1, 3);
        if res.is_ok(){
            web.add("blogs", &res.unwrap().articles);
        }

        let mut sections_hash: HashMap<usize, (Uuid, String, Vec<ArticleBrief>)> = HashMap::new();
        // query category sections
        let cate_sections = Section::query_with_redis_queue(&pg_conn, &redis_pool, "cate_sections");
        if cate_sections.is_ok(){
            let cate_sections_vec = cate_sections.unwrap();
            let cate_sections_len = cate_sections_vec.len();
            web.add("cate_sections_len", &cate_sections_len);
            for (idx, section) in cate_sections_vec.iter().enumerate() {
                let res = Article::query_articles_with_section_id_and_stype_paging(&pg_conn, section.id, 0, 1, 3);
                if res.is_ok(){
                    sections_hash.insert(idx, (section.id, section.title.clone(), res.unwrap().articles));
                }
            }
            web.add("sections_hash", &sections_hash);
        }

        // query project sections
        let mut projects_hash: HashMap<usize, (Uuid, String, Vec<ArticleBrief>)> = HashMap::new();
        // query category sections
        let proj_sections = Section::query_with_redis_queue(&pg_conn, &redis_pool, "proj_sections");
        if proj_sections.is_ok(){
            let proj_sections_vec = proj_sections.unwrap();
            web.add("proj_sections_len", &proj_sections_vec.len());
            for (idx, section) in proj_sections_vec.iter().enumerate() {
                let res = Article::query_articles_with_section_id_and_stype_paging(&pg_conn, section.id, 0, 1, 3);
                if res.is_ok(){
                    projects_hash.insert(idx, (section.id, section.title.clone(), res.unwrap().articles));
                }
            }
            web.add("projects_hash", &projects_hash);
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
