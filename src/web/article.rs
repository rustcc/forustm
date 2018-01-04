use super::super::{Article, Permissions, RUser, WebContext};
use super::super::Postgresql;
use super::super::models::CommentWithNickName;
use super::super::page_size;
use sapper::{Request, Response, Result as SapperResult, SapperModule, SapperRouter};
use sapper_std::{render, PathParams};
use uuid::Uuid;

pub struct WebArticle;

impl WebArticle {
    fn article(req: &mut Request) -> SapperResult<Response> {
        let mut web = req.ext().get::<WebContext>().unwrap().clone();

        let params = get_path_params!(req);
        let id: Result<Uuid, _> = t_param!(params, "id").parse();
        if let Err(e) = id {
            return res_400!(format!("UUID invalid: {}", e));
        }

        let pg_conn = req.ext().get::<Postgresql>().unwrap().get().unwrap();
        let id = id.unwrap();
        let res = Article::query_article(&pg_conn, id);
        match res {
            Ok(r) => {
                // article
                web.add("res", &r);

                // author
                let author = RUser::query_with_id(&pg_conn, r.author_id).unwrap();
                web.add("author", &author);

                // comments
                let page = 1;
                let comments = CommentWithNickName::comments_with_article_id_paging(&pg_conn, id, page, page_size());
                match comments {
                    Ok(com) => {
                        web.add("page_size", &page_size());
                        web.add("page", &page);

                        web.add("comments", &com.comments);
                        web.add("total", &com.total);
                        web.add("max_page", &com.max_page);

                        res_html!("detailArticle.html", web)
                    }
                    Err(e) => res_500!(e),
                }
            }
            Err(e) => res_400!(format!("article not found: {}", e)),
        }
    }

    fn edit(req: &mut Request) -> SapperResult<Response> {
        let web = req.ext().get::<WebContext>().unwrap().clone();
        match *req.ext().get::<Permissions>().unwrap() {
            Some(_) => res_html!("editArticle.html", web),
            None => res_redirect!("/login"),
        }
    }
}

impl SapperModule for WebArticle {
    fn router(&self, router: &mut SapperRouter) -> SapperResult<()> {
        router.get("/user/article/edit", WebArticle::edit);
        router.get("/article/:id", WebArticle::article);

        Ok(())
    }
}
