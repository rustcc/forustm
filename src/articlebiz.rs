use sapper::Result;
use sapper::Request;
use sapper::Response;
use sapper::SapperRouter;
use sapper::SapperModule;

use std::str;
use sapper::header::ContentType;
use sapper_std::{FormParams, QueryParams};

use chrono::prelude::*;

use dbconn::get_conn;
use models::{RUser, NewRUser, Article, NewArticle};
use schema::article::table as article_table;

use diesel;
use diesel::prelude::*;
use serde_json;


#[derive(Clone)]
pub struct ArticleBiz;

impl ArticleBiz {
    fn new_article(req: &mut Request) -> Result<Response> {
        use schema::article::dsl::*;

        // get params
        let form = get_form_params!(req);
        
        let _title = t_param!(form, "title");
        let _content = t_param!(form, "content");
        let _section_id = t_param_parse!(form, "section_id", i32);
        let _author_id = t_param_parse!(form, "author_id", i32);

        // make object
        let new_at = NewArticle {
            title: _title.to_owned(),
            content: _content.to_owned(),
            section_id: _section_id,
            author_id: _author_id,
            created_time: Utc::now().timestamp(),
            tags: "".to_owned()
        };

        // insert to db
        let conn = get_conn();
        let _at = diesel::insert(&new_at).into(article_table)
            .get_result::<Article>(&conn)
            .expect("Error saving new article");
        println!("{:?}", _at);
        
        // return to client
        let json2ret = json!({
            "success": true,
            "article": _at
        });
        res_json!(json2ret)
    }
    
    fn one_article(req: &mut Request) -> Result<Response> {
        // get params
        let query = get_query_params!(req);
        let _at_id = t_param_parse!(query, "id", i32);

        let conn = get_conn();
        match article_table.find(_at_id).first::<Article>(&conn) {
            Ok(at) => {
                let json2ret = json!({
                    "success": true,
                    "article": at
                });

                res_json!(json2ret)
            },
            Err(_) => {
                println!("find 0 row in article table with id: {}", _at_id);
                
                let json2ret = json!({
                    "success": false,
                    "info": format!("no this article, id: {}", _at_id)
                });

                res_json!(json2ret)
            }
        }

    }

    fn list_article(req: &mut Request) -> Result<Response> {
        use schema::article::dsl::*;
        
        // get params
        let query = get_query_params!(req);
        let _section_id = t_param_parse!(query, "section_id", i32);
        // page size
        let ps = t_param_parse!(query, "ps", i32);
        // page number
        let pn = t_param_parse!(query, "pn", i32);

        let conn = get_conn();
        let ats: Vec<Article> = article_table.filter(section_id.eq(_section_id))
                                .order(created_time.desc())
                                .offset((ps * pn) as i64)
                                .limit(ps as i64)
                                .load(&conn).unwrap();
        
        // return to client
        let json2ret = json!({
            "success": true,
            "articles": ats
        });
        res_json!(json2ret)

    }

    // modify title, content
    fn edit_one(req: &mut Request) -> Result<Response> {
        use schema::article::dsl::*;

        // get params
        let form = get_form_params!(req);
        
        let _at_id = t_param_parse!(form, "article_id", i32);
        let _title = t_param!(form, "title");
        let _content = t_param!(form, "content");

        let conn = get_conn();
        match article_table.find(_at_id).first::<Article>(&conn) {
            Ok(at) => {
                let _at = diesel::update(article_table.find(_at_id)).set((
                    title.eq(_title),
                    content.eq(_content)
                )).get_result::<Article>(&conn).unwrap();
                
                let json2ret = json!({
                    "success": true,
                    "article": _at
                });

                res_json!(json2ret)
            },
            Err(_) => {
                println!("find 0 row in article table with id: {}", _at_id);
                
                let json2ret = json!({
                    "success": true,
                    "info": format!("no this article, id: {}", _at_id)
                });

                res_json!(json2ret)
            }
        }
    }

    fn delete_one(req: &mut Request) -> Result<Response> {
        use schema::article::dsl::*;

        // get params
        let form = get_form_params!(req);
        
        let _at_id = t_param_parse!(form, "article_id", i32);

        let conn = get_conn();
        let num_deleted = diesel::delete(article_table.find(_at_id))
                                .execute(&conn)
                                .expect("Error deleting article");

        // here, we don't delete associated comments on this article

        // return 
        let json2ret = json!({
            "success": true,
            "num_deleted": num_deleted
        });

        res_json!(json2ret)

    }

}

// set before, after middleware, and add routers
impl SapperModule for ArticleBiz {
    
    fn before(&self, req: &mut Request) -> Result<()> {
        Ok(())
    }
    
    fn after(&self, req: &Request, res: &mut Response) -> Result<()> {
        Ok(())
    }
    
    fn router(&self, router: &mut SapperRouter) -> Result<()> {
        
        router.post("/api/v1/article/new", ArticleBiz::new_article);
        router.get("/api/v1/article/one", ArticleBiz::one_article);
        router.get("/api/v1/article/list", ArticleBiz::list_article);
        
        router.post("/api/v1/article/edit_one", ArticleBiz::edit_one);
        router.post("/api/v1/article/delete_one", ArticleBiz::delete_one);
        
        Ok(())
        
    }
    
    
}

