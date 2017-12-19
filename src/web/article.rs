use sapper::{SapperModule, SapperRouter, Response, Request, Result as SapperResult};
use sapper_std::{Context, render, PathParams};
use super::super::{Postgresql};
use super::super::{Article, RUser, Comment};
use uuid::Uuid;

pub struct WebArticle;

impl WebArticle {
    fn article(req: &mut Request) -> SapperResult<Response> {
        let mut web = Context::new();
        let pg_conn = req.ext().get::<Postgresql>().unwrap().get().unwrap();

        let params = get_path_params!(req);
        let id: Result<Uuid, _> = t_param!(params, "id").clone().parse();
        if let Err(e) = id {
            return res_400!(format!("UUID invalid: {}", e));
        }

        let id = id.unwrap();
        let res = Article::query_article(&pg_conn, id);
        match res {
            Ok(r) => {
                // article
                web.add("res", &r);

                // author
                let manager = RUser::query_with_id(&pg_conn, r.author_id).unwrap();
                web.add("manager", &manager);

                // comments
                let page = 1;
                let comments = Comment::comments_with_article_id_paging(&pg_conn, id, page, 20);
                match comments {
                    Ok(coms) => {
                        web.add("page", &page);

                        web.add("comments", &coms.comments);
                        web.add("total", &coms.total);
                        web.add("max_page", &coms.max_page);

                        println!("{:?}", coms);
                        res_html!("detailArticle.html", web)
                    },
                    Err(e) => res_500!(e),
                }
            }
            Err(e) => res_400!(format!("article not found: {}", e)),
        }
    }
}

impl SapperModule for WebArticle {
    fn router(&self, router: &mut SapperRouter) -> SapperResult<()> {
        router.get("/article/:id", WebArticle::article);

        Ok(())
    }
}
