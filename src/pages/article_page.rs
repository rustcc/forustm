use sapper::Result;
use sapper::Request;
use sapper::Response;
use sapper::SapperRouter;
use sapper::SapperModule;

use std::str;
use sapper::header::ContentType;
use sapper_std::{FormParams, QueryParams, PathParams, Context, render};
use sapper::Client;

use chrono::prelude::*;

use models::{RUser, Article};
use serde_json;
use serde_json::{Value, Error};



#[derive(Clone)]
pub struct ArticlePage;

impl ArticlePage {
    fn new_article(req: &mut Request) -> Result<Response> {
               
        let mut c = Context::new();
        res_html!("article_new.html", c)
    }
    
    fn one_article(req: &mut Request) -> Result<Response> {
        // get article id param
        let params = get_path_params!(req); 
        let aid = t_param!(params, "aid");

        // define returned data structure
        #[derive(Serialize, Deserialize)]
        struct RetData {
            success: bool,
            article: Article
        }

        let ret: Option<RetData> = get_url!(
                "/api/v1/article/one", 
                &("id=".to_owned() + aid));
        if ret.is_some() {
            let ret_obj = ret.unwrap();
            // render
            let mut c = Context::new();
            c.add("article", &ret_obj.article);
            res_html!("article.html", c)
        }
        else {
            // here, we should process the error case, no data
            // res_404!()
            res_html!("404.html", Context::new())
        }

    }

    fn edit_article(req: &mut Request) -> Result<Response> {
        // get article id param
        let params = get_path_params!(req); 
        let aid = t_param!(params, "aid");
        
        // define returned data structure
        #[derive(Serialize, Deserialize)]
        struct RetData {
            success: bool,
            article: Article
        }

        let ret: Option<RetData> = get_url!(
                "/api/v1/article/one", 
                &("id=".to_owned() + aid));
        if ret.is_some() {
            let ret_obj = ret.unwrap();
            // render
            let mut c = Context::new();
            c.add("article", &ret_obj.article);
            res_html!("article_new.html", c)
        }
        else {
            // here, we should process the error case, no data
            // res_404!()
            res_html!("404.html", Context::new())
        }

    }

    fn list_article(req: &mut Request) -> Result<Response> {
        // get article id param
        let params = get_path_params!(req); 
        let sec = t_param!(params, "sec");
        let pn = t_param!(params, "pn");

        // define returned data structure
        #[derive(Serialize, Deserialize)]
        struct RetData {
            success: bool,
            articles: Vec<Article>
        }

        let ret: Option<RetData> = get_url!(
                "/api/v1/article/list", 
                &("section_id=".to_owned() + sec
                  + "&ps=10"
                  + "&pn=" + pn
                  ));
        if ret.is_some() {
            let ret_obj = ret.unwrap();
            // render
            let mut c = Context::new();
            c.add("articles", &ret_obj.articles);
            res_html!("article_list.html", c)
        }
        else {
            // here, we should process the error case, no data
            // res_404!()
            res_html!("404.html", Context::new())
        }

    }

}

// set before, after middleware, and add routers
impl SapperModule for ArticlePage {
    
    fn before(&self, req: &mut Request) -> Result<()> {
        Ok(())
    }
    
    fn after(&self, req: &Request, res: &mut Response) -> Result<()> {
        Ok(())
    }
    
    fn router(&self, router: &mut SapperRouter) -> Result<()> {
        
        router.get("/article/new", ArticlePage::new_article);
        router.get("/article/edit/:aid", ArticlePage::edit_article);
        router.get("/article/:aid", ArticlePage::one_article);
        router.get("/article/list/:sec", ArticlePage::list_article);
        
        Ok(())
    }
}

