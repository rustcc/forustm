use sapper::{SapperModule, SapperRouter, Response, Request, Result as SapperResult};
use sapper_std::{render, PathParams, SessionVal};
use super::super::{Postgresql, Redis, WebContext, Permissions};
use super::super::{Section, Article, RUser};
use uuid::Uuid;
use serde_json;

pub struct WebSection;

enum SectionTypes {
    Section = 0,
    Blog = 1,
}

impl WebSection {
    fn section(req: &mut Request) -> SapperResult<Response> {
        let mut web = req.ext().get::<WebContext>().unwrap().clone();
        let pg_conn = req.ext().get::<Postgresql>().unwrap().get().unwrap();

        let path_params = get_path_params!(req);

        let id: Uuid = match t_param!(path_params, "id").clone().parse() {
            Ok(i) => i,
            Err(err) => return res_400!(format!("UUID invalid: {}", err)),
        };
        let page = 1i64;
        web.add("id", &id);
        web.add("page", &page);

        // add permission
        let identify = req.ext().get::<Permissions>().unwrap();
        match *identify {
            Some(i) => {
                let cookie = req.ext().get::<SessionVal>().unwrap();
                let redis_pool = req.ext().get::<Redis>().unwrap();
                let user: RUser = serde_json::from_str(&RUser::view_with_cookie(redis_pool, cookie)).unwrap();
                web.add("user", &user);
                web.add("identify", &i);
            }
            None => {
                web.add("identify", &-1);
            }
        }

        let res = Section::query_with_section_id(&pg_conn, id);

        match res {
            Ok(r) => {
                if r.stype != SectionTypes::Section as i32 {
                    // return res_400!(format!("section not found {}, it's type is blog", r.id))
                    return res_redirect!(format!("/blog/{}", r.suser.unwrap()));
                }

                web.add("res", &r);

                let articles = Article::query_articles_with_section_id_paging(&pg_conn, id, page, 20);
                match articles {
                    Ok(arts) => {
                        //println!("articles: {:?}", &arts);
                        web.add("articles", &arts.articles);
                        web.add("total", &arts.total);
                        web.add("max_page", &arts.max_page);

                        if let Some(suid) = r.suser {
                            let manager = RUser::query_with_id(&pg_conn, suid).unwrap();
                            web.add("manager", &manager);
                        } else {
                            web.add("manager", &false);
                        }

                        res_html!("detailSection.html", web)
                    }
                    Err(e) => res_500!(e),
                }
            }
            Err(e) => res_500!(format!("section not found: {}", e)),
        }
    }

    fn blog(req: &mut Request) -> SapperResult<Response> {
        let mut web = req.ext().get::<WebContext>().unwrap().clone();
        let pg_conn = req.ext().get::<Postgresql>().unwrap().get().unwrap();

        let path_params = get_path_params!(req);

        let id: Uuid = match t_param!(path_params, "id").clone().parse() {
            Ok(i) => i,
            Err(err) => return res_400!(format!("UUID invalid: {}", err)),
        };
        let page = 1i64;
        web.add("id", &id);
        web.add("page", &page);

        // add permission
        let permission = match *req.ext().get::<Permissions>().unwrap() {
            Some(n) => n,
            None => 9,
        };
        web.add("permission", &permission);

        let res = Section::query_with_user_id(&pg_conn, id);
        match res {
            Ok(r) => {
                if r.stype != SectionTypes::Blog as i32 {
                    // return res_400!(format!("section not found {}, it's type is section", r.id))
                    return res_redirect!(format!("/section/{}", r.id));
                }

                web.add("res", &r);

                let articles = Article::query_articles_with_section_id_paging(&pg_conn, r.id, page, 20);
                match articles {
                    Ok(arts) => {
                        //println!("articles: {:?}", &arts);
                        web.add("articles", &arts.articles);
                        web.add("total", &arts.total);
                        web.add("max_page", &arts.max_page);

                        if let Some(suid) = r.suser {
                            let manager = RUser::query_with_id(&pg_conn, suid).unwrap();
                            web.add("manager", &manager);
                        }

                        res_html!("detailSection.html", web)
                    }
                    Err(e) => res_500!(e),
                }
            }
            Err(e) => res_500!(format!("section not found: {}", e)),
        }
    }
    fn blogs(req: &mut Request) -> SapperResult<Response> {
        let mut web = req.ext().get::<WebContext>().unwrap().clone();
        let pg_conn = req.ext().get::<Postgresql>().unwrap().get().unwrap();
        let page = 1;
        let res = Article::query_articles_by_stype_paging(&pg_conn, 1, page, 2);

        match res {
            Ok(r) => {
                web.add("articles", &r.articles);
                web.add("page", &page);
                web.add("total", &r.total);
                web.add("max_page", &r.max_page);
                res_html!("blogs.html", web)
            },
            Err(e) => res_400!(format!("blogs not found: {}", e)),
        }

    }
}

impl SapperModule for WebSection {
    fn router(&self, router: &mut SapperRouter) -> SapperResult<()> {
        router.get("/section/:id", WebSection::section);
        router.get("/blog/:id", WebSection::blog);
        router.get("/blogs", WebSection::blogs);

        Ok(())
    }
}
