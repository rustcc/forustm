use diesel::types::*;

use super::schema::article;
#[derive(Queryable, Debug, Serialize, Deserialize)]
pub struct Article {
    pub id: i32,
    pub title: String,
    pub content: String,
    pub section_id: i32,
    pub author_id: i32,
    pub tags: String,
    pub created_time: i64,
}

#[derive(Insertable, Debug, Serialize, Deserialize)]
#[table_name="article"]
pub struct NewArticle {
    pub title: String,
    pub content: String,
    pub section_id: i32,
    pub author_id: i32,
    pub tags: String,
    pub created_time: i64,
}


use super::schema::ruser;
#[derive(Queryable, Debug, Serialize, Deserialize)]
pub struct RUser {
    pub id: i32,
    pub account: String,
    pub password: String,
    pub salt: String,
    pub nickname: String,
    pub avatar: String,
    pub wx_openid: String,
    pub say: String,
    pub signup_time: i64,
}

#[derive(Insertable, Debug, Serialize, Deserialize)]
#[table_name="ruser"]
pub struct NewRUser {
    pub account: String,
    pub password: String,
    pub salt: String,
    pub nickname: String,
    pub avatar: String,
    pub wx_openid: String,
    pub say: String,
    pub signup_time: i64,
}


use super::schema::comment;
#[derive(Queryable, Debug, Serialize, Deserialize)]
pub struct Comment {
    pub id: i32,
    pub content: String,
    pub article_id: i32,
    pub author_id: i32,
    pub created_time: i64,
}

#[derive(Insertable, Debug, Serialize, Deserialize)]
#[table_name="comment"]
pub struct NewComment {
    pub content: String,
    pub article_id: i32,
    pub author_id: i32,
    pub created_time: i64,
}

use super::schema::section;
#[derive(Queryable, Debug, Serialize, Deserialize)]
pub struct Section {
    pub id: i32,
    pub title: String,
    // stype: 0, forum, 1, blog
    pub stype: i32,
    pub created_time: i64,
}

#[derive(Insertable, Debug, Serialize, Deserialize)]
#[table_name="section"]
pub struct NewSection {
    pub title: String,
    pub stype: i32,
    pub created_time: i64,
}

