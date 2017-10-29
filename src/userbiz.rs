
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
use models::{RUser, NewRUser};
use schema::ruser::table as ruser_table;

use diesel;
use diesel::prelude::*;
use serde_json;
use rand::{thread_rng, Rng};
use md5;

// helper function
fn random_string(length: usize) -> String {
    let s: String = thread_rng().gen_ascii_chars().take(length).collect();
    s 
}

fn md5encode(s: String) -> String {
    let digest = md5::compute(s);
    format!("{:x}", digest)
}

#[derive(Clone)]
pub struct RUserBiz;

impl RUserBiz {
    fn new_user(req: &mut Request) -> Result<Response> {
        use schema::ruser::dsl::*;

        // get params
        let form = get_form_params!(req);
        
        let _account = t_param!(form, "account");
        let _password = t_param!(form, "password");
        let _nickname = t_param!(form, "nickname");

        // make 6 byte salt random string
        let _salt = random_string(6);

        // md5 _password + salt, 
        let encoded_password = md5encode(_password.to_owned() + &_salt);

        // make object
        let new_ruser = NewRUser {
            account: _account.to_owned(),
            nickname: _nickname.to_owned(),
            password: encoded_password,
            salt: _salt,
            avatar: "".to_string(),
            wx_openid: "".to_string(),
            say: "".to_string(),
            signup_time: Utc::now().timestamp(),
        };

        // insert to db
        let conn = get_conn();
        let _user = diesel::insert(&new_ruser).into(ruser_table)
            .get_result::<RUser>(&conn)
            .expect("Error saving new user");
        println!("{:?}", _user);
        
        // return to client
        let json2ret = json!({
            "success": true,
            "user": _user
        });
        res_json!(json2ret)
    }
    
    fn one_user(req: &mut Request) -> Result<Response> {
        // get params
        let query = get_query_params!(req);
        let user_id = t_param_parse!(query, "id", i32);

        let conn = get_conn();
        match ruser_table.find(user_id).first::<RUser>(&conn) {
            Ok(user) => {
                let json2ret = json!({
                    "success": true,
                    "user": user 
                });

                res_json!(json2ret)
            },
            Err(_) => {
                println!("find 0 row in user table with id: {}", user_id);
                
                let json2ret = json!({
                    "success": false,
                    "info": format!("no this user, id: {}", user_id)
                });

                res_json!(json2ret)
            }
        }

    }

    // modify nickname, say, avatar 
    fn edit_one(req: &mut Request) -> Result<Response> {
        use schema::ruser::dsl::*;

        // get params
        let form = get_form_params!(req);
        
        let _user_id = t_param_parse!(form, "user_id", i32);
        let _nickname = t_param!(form, "nickname");
        let _say = t_param!(form, "say");
        let _avatar = t_param!(form, "avatar");

        let conn = get_conn();
        match ruser_table.find(_user_id).first::<RUser>(&conn) {
            Ok(user) => {
                let _user = diesel::update(ruser_table.find(_user_id)).set((
                    nickname.eq(_nickname),
                    say.eq(_say),
                    avatar.eq(_avatar)
                )).get_result::<RUser>(&conn).unwrap();
                
                let json2ret = json!({
                    "success": true,
                    "user": _user
                });

                res_json!(json2ret)
            },
            Err(_) => {
                println!("find 0 row in user table with id: {}", _user_id);
                
                let json2ret = json!({
                    "success": true,
                    "info": format!("no this user, id: {}", _user_id)
                });

                res_json!(json2ret)
            }
        }
    }

    fn change_pwd(req: &mut Request) -> Result<Response> {
        use schema::ruser::dsl::*;

        // get params
        let form = get_form_params!(req);
        
        let _user_id = t_param_parse!(form, "user_id", i32);
        let _password = t_param!(form, "password");
        let _password_new = t_param!(form, "password_new");

        let conn = get_conn();
        match ruser_table.find(_user_id).first::<RUser>(&conn) {
            Ok(user) => {
                // make 6 byte salt random string
                let _salt = random_string(6);

                // md5 _password + salt, 
                let encoded_password = md5encode(_password.to_owned() + &_salt);
                
                let _user = diesel::update(ruser_table.find(_user_id)).set((
                    salt.eq(&_salt),
                    password.eq(&encoded_password)
                )).get_result::<RUser>(&conn).unwrap();
                
                let json2ret = json!({
                    "success": true,
                    "user": _user
                });

                res_json!(json2ret)
            },
            Err(_) => {
                println!("find 0 row in user table with id: {}", _user_id);
                
                let json2ret = json!({
                    "success": true,
                    "info": format!("no this user, id: {}", _user_id)
                });

                res_json!(json2ret)
            }
        }
    }

}

// set before, after middleware, and add routers
impl SapperModule for RUserBiz {
    
    fn before(&self, req: &mut Request) -> Result<()> {
        Ok(())
    }
    
    fn after(&self, req: &Request, res: &mut Response) -> Result<()> {
        Ok(())
    }
    
    fn router(&self, router: &mut SapperRouter) -> Result<()> {
        
        router.post("/api/v1/user/new", RUserBiz::new_user);
        router.get("/api/v1/user/one", RUserBiz::one_user);
        
        router.post("/api/v1/user/edit_one", RUserBiz::edit_one);
        router.post("/api/v1/user/change_pwd", RUserBiz::change_pwd);
        
        Ok(())
        
    }
    
    
}

