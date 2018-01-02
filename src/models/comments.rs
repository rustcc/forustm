use super::super::{markdown_render, RUser, RedisPool};
use super::super::{comment, ruser};
use super::super::comment::dsl::comment as all_comments;
use super::super::ruser::dsl::ruser as all_rusers;

use chrono::NaiveDateTime;
use diesel;
use diesel::PgConnection;
use diesel::prelude::*;
use serde_json;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Queryable, Debug, Clone, Deserialize, Serialize)]
pub struct CommentWithNickName {
    id: Uuid,
    content: String,
    article_id: Uuid,
    author_id: Uuid,
    created_time: NaiveDateTime,
    status: i16, // 0 normal, 1 frozen, 2 deleted

    nickname: String,
}

#[derive(Debug)]
pub struct CommentsWithTotal<T> {
    pub comments: Vec<T>,
    pub total: i64,
    pub max_page: i64,
}

impl CommentWithNickName {
    pub fn query(conn: &PgConnection, limit: i64, offset: i64, article_id: Uuid) -> Result<Vec<Self>, String> {
        let res = all_comments
            .inner_join(all_rusers.on(comment::author_id.eq(ruser::id)))
            .select((
                comment::id,
                comment::content,
                comment::article_id,
                comment::author_id,
                comment::created_time,
                comment::status,
                ruser::nickname,
            ))
            .filter(comment::status.eq(0))
            .filter(comment::article_id.eq(article_id))
            .order(comment::created_time)
            .limit(limit)
            .offset(offset)
            .get_results::<CommentWithNickName>(conn);

        match res {
            Ok(data) => Ok(data),
            Err(err) => Err(format!("{}", err)),
        }
    }

    pub fn comments_with_article_id_paging(
        conn: &PgConnection,
        id: Uuid,
        page: i64,
        page_size: i64,
    ) -> Result<CommentsWithTotal<Self>, String> {
        let _res = all_comments
            .filter(comment::article_id.eq(id))
            .filter(comment::status.eq(0));

        let res = _res.inner_join(all_rusers.on(comment::author_id.eq(ruser::id)))
            .select((
                comment::id,
                comment::content,
                comment::article_id,
                comment::author_id,
                comment::created_time,
                comment::status,
                ruser::nickname,
            ))
            .order(comment::created_time)
            .offset(page_size * (page - 1) as i64)
            .limit(page_size)
            .get_results::<Self>(conn);

        let all_count: i64 = _res.count().get_result(conn).unwrap();

        match res {
            Ok(data) => Ok(CommentsWithTotal {
                comments: data,
                total: all_count,
                max_page: (all_count as f64 / page_size as f64).ceil() as i64,
            }),
            Err(err) => Err(format!("{}", err)),
        }
    }

    fn delete_with_comment_id(conn: &PgConnection, id: Uuid) -> bool {
        diesel::update(all_comments.filter(comment::id.eq(id)))
            .set(comment::status.eq(2))
            .execute(conn)
            .is_ok()
    }

    pub fn delete_with_author_id(conn: &PgConnection, id: Uuid) -> bool {
        diesel::update(all_comments.filter(comment::author_id.eq(id)))
            .set(comment::status.eq(2))
            .execute(conn)
            .is_ok()
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
            .execute(conn)
            .is_ok()
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
            author_id: author_id,
        }
    }

    pub fn insert(self, conn: &PgConnection, redis_pool: &Arc<RedisPool>, cookie: &str) -> bool {
        let info = serde_json::from_str::<RUser>(&redis_pool.hget::<String>(cookie, "info")).unwrap();
        self.into_insert_comments(info.id).insert(conn)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DeleteComment {
    comment_id: Uuid,
    author_id: Uuid,
}

impl DeleteComment {
    pub fn delete(
        self,
        conn: &PgConnection,
        redis_pool: &Arc<RedisPool>,
        cookie: &str,
        permission: &Option<i16>,
    ) -> bool {
        match *permission {
            Some(0) | Some(1) => CommentWithNickName::delete_with_comment_id(conn, self.comment_id),
            _ => {
                let info = serde_json::from_str::<RUser>(&redis_pool.hget::<String>(cookie, "info")).unwrap();
                if self.author_id == info.id {
                    CommentWithNickName::delete_with_comment_id(conn, self.comment_id)
                } else {
                    false
                }
            }
        }
    }
}
