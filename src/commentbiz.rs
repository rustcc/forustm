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
use models::{Comment, NewComment};
use schema::comment::table as comment_table;

use diesel;
use diesel::prelude::*;
use serde_json;


#[derive(Clone)]
pub struct CommentBiz;

impl CommentBiz {
    fn new_comment(req: &mut Request) -> Result<Response> {
        use schema::comment::dsl::*;

        // get params
        let form = get_form_params!(req);
        
        let _content = t_param!(form, "content");
        let _article_id = t_param_parse!(form, "article_id", i32);
        let _author_id = t_param_parse!(form, "author_id", i32);

        // make object
        let new_cm = NewComment {
            content: _content.to_owned(),
            article_id: _article_id,
            author_id: _author_id,
            created_time: Utc::now().timestamp()
        };

        // insert to db
        let conn = get_conn();
        let _cm = diesel::insert(&new_cm).into(comment_table)
            .get_result::<Comment>(&conn)
            .expect("Error saving new comment");
        println!("{:?}", _cm);
        
        // return to client
        let json2ret = json!({
            "success": true,
            "comment": _cm
        });
        res_json!(json2ret)
    }
    
    fn list_comment(req: &mut Request) -> Result<Response> {
        use schema::comment::dsl::*;

        // get params
        let query = get_query_params!(req);
        let _article_id = t_param_parse!(query, "article_id", i32);
        // page size
        let ps = t_param_parse!(query, "ps", i32);
        // page number
        let pn = t_param_parse!(query, "pn", i32);

        let conn = get_conn();
        let cms: Vec<Comment> = comment_table.filter(article_id.eq(_article_id))
                                .order(created_time.desc())
                                .offset((ps * pn) as i64)
                                .limit(ps as i64)
                                .load(&conn).unwrap();
        
        // return to client
        let json2ret = json!({
            "success": true,
            "comments": cms
        });
        res_json!(json2ret)

    }


    fn delete_one(req: &mut Request) -> Result<Response> {
        use schema::comment::dsl::*;

        // get params
        let form = get_form_params!(req);
        
        let _cm_id = t_param_parse!(form, "comment_id", i32);

        let conn = get_conn();
        let num_deleted = diesel::delete(comment_table.find(_cm_id))
                                .execute(&conn)
                                .expect("Error deleting comment");

        // return 
        let json2ret = json!({
            "success": true,
            "num_deleted": num_deleted
        });

        res_json!(json2ret)
    }

}

// set before, after middleware, and add routers
impl SapperModule for CommentBiz {
    
    fn before(&self, req: &mut Request) -> Result<()> {
        Ok(())
    }
    
    fn after(&self, req: &Request, res: &mut Response) -> Result<()> {
        Ok(())
    }
    
    fn router(&self, router: &mut SapperRouter) -> Result<()> {
        
        router.post("/api/v1/comment/new", CommentBiz::new_comment);
        router.get("/api/v1/comment/list", CommentBiz::list_comment);
        
        router.post("/api/v1/comment/delete_one", CommentBiz::delete_one);
        
        Ok(())
        
    }
    
    
}

