use sapper::{ SapperModule, SapperRouter, Response, Request, Result as SapperResult };
use sapper_std::{ Context, render, SessionVal, PathParams };
use super::super::{ Redis, Postgresql };
use super::super::{ Section, InsertSection, Articles };
use uuid::Uuid;

pub struct WebSection;

impl WebSection {
    fn index(req: &mut Request) -> SapperResult<Response> {
        let mut web = Context::new();
        let pg_conn = req.ext().get::<Postgresql>().unwrap().get().unwrap();

        let params = get_path_params!(req);
        let id: Result<Uuid, _> = t_param!(params, "id").clone().parse();
        if let Err(e) = id {
            return res_400!(format!("UUID invalid: {}", e));
        }

        let id = id.unwrap();
        let res = Section::query_by_id(&pg_conn, id);
        match res {
            Ok(r) => {
                web.add("res", &r);
                let articles = Articles::query_articles_with_section_id(&pg_conn, id);
                match articles {
                    Ok(arts) => {
                        //println!("articles: {:?}", &arts);
                        web.add("articles", &arts);
                        res_html!("detailSection.html", web)
                    }
                    Err(e) => {
                        res_500!(e)
                    }
                }
            }
            Err(e) => {
                res_500!(format!("section not found: {}", e))
            }
        }
    }

    fn article(req: &mut Request) -> SapperResult<Response> {
        let mut web = Context::new();
        let pg_conn = req.ext().get::<Postgresql>().unwrap().get().unwrap();

        let params = get_path_params!(req);
        let id: Result<Uuid, _> = t_param!(params, "id").clone().parse();
        if let Err(e) = id {
            return res_400!(format!("UUID invalid: {}", e));
        }

        let id = id.unwrap();
        let res = Section::query_by_id(&pg_conn, id);
        match res {
            Ok(r) => {
                web.add("res", &r);
                let articles = Articles::query_articles_with_section_id(&pg_conn, id);
                match articles {
                    Ok(arts) => {
                        //println!("articles: {:?}", &arts);
                        web.add("articles", &arts);
                        res_html!("detailArticle.html", web)
                    }
                    Err(e) => {
                        res_500!(e)
                    }
                }
            }
            Err(e) => {
                res_500!(format!("section not found: {}", e))
            }
        }
    }
}

impl SapperModule for WebSection {
    fn router(&self, router: &mut SapperRouter) -> SapperResult<()> {
        router.get("/section/:id", WebSection::index);
        router.get("/article/:id", WebSection::article);

        Ok(())
    }
}
