use sapper::{SapperModule, SapperRouter, Response, Request, Result as SapperResult};
use sapper_std::{render, PathParams, SessionVal};
use super::super::{Postgresql, Redis};
use super::super::{Article, RUser, Permissions, WebContext};
use super::super::models::CommentWithNickName;
use uuid::Uuid;
use serde_json;
pub struct WebArticle;

impl WebArticle {
    fn article(req: &mut Request) -> SapperResult<Response> {
        let mut web = req.ext().get::<WebContext>().unwrap().clone();

        let params = get_path_params!(req);
        let id: Result<Uuid, _> = t_param!(params, "id").clone().parse();
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
                let comments = CommentWithNickName::comments_with_article_id_paging(&pg_conn, id, page, 20);
                match comments {
                    Ok(com) => {
                        web.add("page", &page);

                        web.add("comments", &com.comments);
                        web.add("total", &com.total);
                        web.add("max_page", &com.max_page);

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

                        res_html!("detailArticle.html", web)
                    },
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
            None => res_redirect!("/login")
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
