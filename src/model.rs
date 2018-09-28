//! =============================================================
//! All structs defined here must keep the field order as the
//! same as db table schema definitions.
//! =============================================================



///
/// Model: User
/// Db table: ruser
///

#[derive(Queryable)]
struct User {
    
    pub id: Uuid,
    
    // use email defaultly
    pub account: String,
    
    pub password: String,
    
    pub salt: String,
    
    pub nickname: String,
    
    pub avatar: Option<String>,
    
    pub wx_openid: Option<String>,
    
    pub say: Option<String>,
   
    // user signup time
    pub signup_time: NaiveDateTime,
    
    // user role: member => 2, manager => 1, admin => 0
    pub role: i16,
    
    // user status: 0 normal, 1 frozen, 2 deleted
    pub status: i16,
    
    pub github: Option<String>,
}

/// 
/// Model: Section
/// DB table: section
///

#[derive(Queryable)]
pub struct Section {
    
    pub id: Uuid,
    
    pub title: String,
    
    pub description: String,
    
    // use stype to separate forum section and user blog section
    // 0 section, 1 user blog
    pub stype: i32,
   
    // if stype==1, record the binding user to section
    pub suser: Option<Uuid>,
    
    pub created_time: NaiveDateTime,
    
    // 0 normal, 1 frozen, 2 deleted
    pub status: i16, 
}

/// 
/// Model: Article
/// DB table: article
///

#[derive(Queryable)]
struct Article {

    pub id: Uuid,

    pub title: String,

    pub raw_content: String,

    pub content: String,

    pub section_id: Uuid,

    pub author_id: Uuid,
    
    pub tags: String,
    
    // used to planet order ranking: 0 section, 1 user blog
    pub stype: i32,

    pub created_time: NaiveDateTime,
    
    // 0 normal, 1 frozen, 2 deleted
    pub status: i16,

}


/// 
/// Model: Comment
/// DB table: comment
///

#[derive(Queryable)]
pub struct Comment {
    
    pub id: Uuid,
    
    pub content: String,
    
    pub article_id: Uuid,
    
    pub author_id: Uuid,
    
    pub created_time: NaiveDateTime,
    
    // 0 normal, 1 frozen, 2 deleted
    pub status: i16,

}


/// 
/// Model: ArticleStats
/// DB table: article_stats
///

#[derive(Queryable)]
pub struct ArticleStats {
    
    pub id: Uuid,
    
    pub article_id: Uuid,
    
    pub created_time: NaiveDateTime,
    
    pub ruser_id: Option<Uuid>,
    
    pub user_agent: Option<String>,
    
    pub visitor_ip: Option<String>,
}


/// 
/// Model: UserNotify
/// DB: redis
/// a cached user notifications queue
///

#[derive(Queryable)]
pub struct UserNotify {
    
    pub user_id: Uuid,
    
    pub send_user_name: String,
    
    pub article_id: Uuid,
    
    pub article_title: String,
    
    pub notify_type: String,
}


