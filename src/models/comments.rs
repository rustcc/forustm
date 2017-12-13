use super::super::comment::dsl::comment as all_comments;
use super::super::comment;
use super::super::{ markdown_render, RedisPool, RUser };

use uuid::Uuid;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::PgConnection;
use diesel;
use std::sync::Arc;
use serde_json;


#[derive(Queryable, Debug, Clone, Deserialize, Serialize)]
pub struct Comment {
    id: Uuid,
    content: String,
    article_id: Uuid,
    author_id: Uuid,
    created_time: NaiveDateTime
}

impl Comment {
    pub fn query(conn: &PgConnection, limit: i64, offset: i64, article_id: Uuid) -> Result<Vec<Self>, String> {
        let res = all_comments.filter(comment::article_id.eq(article_id))
            .order(comment::created_time)
            .limit(limit)
            .offset(offset)
            .get_results::<Self>(conn);
        match res {
            Ok(data) => {
                Ok(data)
            },
            Err(err) => Err(format!("{}", err))
        }
    }

    pub fn delete_with_comment_id(conn: &PgConnection, id: Uuid) -> bool {
        diesel::delete(all_comments.filter(comment::id.eq(id)))
            .execute(conn).is_ok()
    }
    pub fn delete_with_author_id(conn: &PgConnection, id: Uuid) -> bool {
        diesel::delete(all_comments.filter(comment::author_id.eq(id)))
            .execute(conn).is_ok()
    }
}

#[derive(Insertable, Debug, Clone)]
#[table_name = "comment"]
struct InsertComment {
    content: String,
    article_id: Uuid,
    author_id: Uuid,
}

impl InsertComment {
    fn insert(self, conn: &PgConnection) -> bool {
        diesel::insert_into(comment::table)
            .values(&self)
            .execute(conn).is_ok()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NewComment {
    content: String,
    article_id: Uuid,
}

impl NewComment {
    fn into_insert_comments(self, author_id: Uuid) -> InsertComment {
        InsertComment {
            content: markdown_render(&self.content),
            article_id: self.article_id,
            author_id,
        }
    }

    pub fn insert(self, conn: &PgConnection, redis_pool: &Arc<RedisPool>, cookie: &str, admin: &bool) -> bool {
        let redis_key = match admin {
            &true => { "admin_".to_string() + cookie }
            &false => { "user_".to_string() + cookie }
        };
        let info = serde_json::from_str::<RUser>(&redis_pool.hget::<String>(&redis_key, "info")).unwrap();
        self.into_insert_comments(info.id).insert(conn)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DeleteComment {
    comment_id: Uuid,
    author_id: Uuid
}

impl DeleteComment {
    pub fn delete(self, conn: &PgConnection, redis_pool: &Arc<RedisPool>, cookie: &str, admin: &i16) -> bool {
        match admin {
            &0 => {
                Comment::delete_with_comment_id(conn, self.comment_id)
            }
            _ => {
                let redis_key = "user_".to_string() + cookie;
                let info = serde_json::from_str::<RUser>(&redis_pool.hget::<String>(&redis_key, "info")).unwrap();
                match self.author_id == info.id {
                    true => Comment::delete_with_comment_id(conn, self.comment_id),
                    false => false
                }
            }
        }
    }
}
