extern crate sapper;
extern crate serde;
#[macro_use] extern crate serde_json;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate log;
#[macro_use] extern crate sapper_std;
#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_codegen;
extern crate dotenv;
extern crate chrono;
extern crate rand;
extern crate md5;

use sapper::{SapperApp, SapperAppShell, Request, Response, Result, SapperModule};


pub mod dbconn;
pub mod schema;
pub mod models;

mod userbiz;
use userbiz::RUserBiz;
mod articlebiz;
use articlebiz::ArticleBiz;
mod commentbiz;
use commentbiz::CommentBiz;
mod sectionbiz;
use sectionbiz::SectionBiz;


#[derive(Clone)]
struct MyApp;
// total entry and exitice
impl SapperAppShell for MyApp {
    fn before(&self, req: &mut Request) -> Result<()> {
        sapper_std::init(req);       
        Ok(())
    }
    
    fn after(&self, req: &Request, res: &mut Response) -> Result<()> {
        sapper_std::finish(req, res);       
        Ok(())
    }
}



pub fn main() {
    // fn init_global(req: &mut Request) -> Result<()> {
    //     Ok(())
    // }
    
    let mut sapp = SapperApp::new();
    sapp.address("127.0.0.1")
        .port(1337)
        // .init_global(Box::new(init_global))
        .with_shell(Box::new(MyApp))
        .add_module(Box::new(RUserBiz))
        .add_module(Box::new(ArticleBiz))
        .add_module(Box::new(CommentBiz))
        .add_module(Box::new(SectionBiz));
    
    println!("Listening on http://127.0.0.1:1337");
    sapp.run_http();
    
}

