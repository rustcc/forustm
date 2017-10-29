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
use models::{Section, NewSection};
use schema::section::table as section_table;

use diesel;
use diesel::prelude::*;
use serde_json;


#[derive(Clone)]
pub struct SectionBiz;

impl SectionBiz {
    fn new_section(req: &mut Request) -> Result<Response> {
        use schema::section::dsl::*;

        // get params
        let form = get_form_params!(req);
        
        let _title = t_param!(form, "title");
        let _stype = t_param_parse!(form, "stype", i32);

        // make object
        let new_sec = NewSection {
            title: _title.to_owned(),
            stype: _stype,
            created_time: Utc::now().timestamp(),
        };

        // insert to db
        let conn = get_conn();
        let _sec = diesel::insert(&new_sec).into(section_table)
            .get_result::<Section>(&conn)
            .expect("Error saving new section");
        println!("{:?}", _sec);
        
        // return to client
        let json2ret = json!({
            "success": true,
            "user": _sec
        });
        res_json!(json2ret)
    }
    
    fn list_forum_section(req: &mut Request) -> Result<Response> {
        use schema::section::dsl::*;

        // get params
        let query = get_query_params!(req);
        // page size
        let ps = t_param_parse!(query, "ps", i32);
        // page number
        let pn = t_param_parse!(query, "pn", i32);

        let conn = get_conn();
        let secs: Vec<Section> = section_table.filter(stype.eq(0))
                                .order(created_time.asc())
                                .offset((ps * pn) as i64)
                                .limit(ps as i64)
                                .load(&conn).unwrap();
        
        // return to client
        let json2ret = json!({
            "success": true,
            "sections": secs
        });
        res_json!(json2ret)

    }

    // modify title, content
    fn edit_one(req: &mut Request) -> Result<Response> {
        use schema::section::dsl::*;

        // get params
        let form = get_form_params!(req);
        
        let _sec_id = t_param_parse!(form, "section_id", i32);
        let _title = t_param!(form, "title");

        let conn = get_conn();
        match section_table.find(_sec_id).first::<Section>(&conn) {
            Ok(at) => {
                let _sec = diesel::update(section_table.find(_sec_id)).set((
                    title.eq(_title)
                )).get_result::<Section>(&conn).unwrap();
                
                let json2ret = json!({
                    "success": true,
                    "section": _sec
                });

                res_json!(json2ret)
            },
            Err(_) => {
                println!("find 0 row in section table with id: {}", _sec_id);
                
                let json2ret = json!({
                    "success": true,
                    "info": format!("no this section, id: {}", _sec_id)
                });

                res_json!(json2ret)
            }
        }
    }

    fn delete_one(req: &mut Request) -> Result<Response> {
        use schema::section::dsl::*;

        // get params
        let form = get_form_params!(req);
        
        let _sec_id = t_param_parse!(form, "section_id", i32);

        let conn = get_conn();
        let num_deleted = diesel::delete(section_table.find(_sec_id))
                                .execute(&conn)
                                .expect("Error deleting section");

        // return 
        let json2ret = json!({
            "success": true,
            "num_deleted": num_deleted
        });

        res_json!(json2ret)

    }

}

// set before, after middleware, and add routers
impl SapperModule for SectionBiz {
    
    fn before(&self, req: &mut Request) -> Result<()> {
        Ok(())
    }
    
    fn after(&self, req: &Request, res: &mut Response) -> Result<()> {
        Ok(())
    }
    
    fn router(&self, router: &mut SapperRouter) -> Result<()> {
        
        router.post("/api/v1/section/new", SectionBiz::new_section);
        router.get("/api/v1/section/list", SectionBiz::list_forum_section);
        
        router.post("/api/v1/section/edit_one", SectionBiz::edit_one);
        router.post("/api/v1/section/delete_one", SectionBiz::delete_one);
        
        Ok(())
        
    }
    
    
}

