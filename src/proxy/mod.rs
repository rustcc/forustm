
use sapper::{Error as SapperError, Request, Response, Result as SapperResult, SapperModule, SapperRouter};
use sapper_std::{JsonParams, SessionVal};
use serde_json;

use super::super::{LoginUser, Postgresql, RUser, Redis, RegisteredUser};
use super::super::{inner_get_github_nickname_and_address, inner_get_github_token};

pub struct ProxyModule;

impl ProxyModule {
    fn get_github_token(req: &mut Request) -> SapperResult<Response> {
    
    
    }

    fn get_github_nickname_and_address(req: &mut Request) -> SapperResult<Response> {
    
    
    }

    fn get_github_primary_email(req: &mut Request) -> SapperResult<Response> {

        
    }

}

impl SapperModule for User {

    fn router(&self, router: &mut SapperRouter) -> SapperResult<()> {
        router.post("/inner/get_github_token", ProxyModule::get_github_token);

        router.get("/inner/get_github_nickname_and_address", ProxyModule::get_github_nickname_and_address);

        router.get("/inner/get_github_primary_email", ProxyModule::get_github_primary_email);

        Ok(())
    }
}
