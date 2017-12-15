#![recursion_limit="128"]

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_infer_schema;
extern crate dotenv;
extern crate chrono;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate serde;
extern crate sapper;
#[macro_use]
extern crate sapper_std;
extern crate rand;
extern crate tiny_keccak;
extern crate comrak;
extern crate redis;
extern crate r2d2;
extern crate r2d2_redis;
extern crate r2d2_diesel;
extern crate uuid;

pub mod api;
pub mod schema;
pub mod util;
pub mod models;
pub mod web;
pub mod web_wechat;

pub(crate) use util::{ sha3_256_encode, random_string, markdown_render };
pub(crate) use schema::{ article, ruser, section, comment };
pub(crate) use models::{ Articles, EditArticle, NewArticle };
pub(crate) use models::{ RUser, RegisteredUser, LoginUser, ChangePermission, ChangePassword, EditUser };
pub(crate) use models::{ Comment, NewComment, DeleteComment };
pub(crate) use models::{ InsertSection, Section };

pub use util::{ Postgresql, RedisPool, Redis, create_pg_pool, create_redis_pool, get_identity, Permissions};
pub use web::{ Index };
pub use api::{ Visitor };
