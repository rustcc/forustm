use super::RUserDto;
use super::super::util::markdown_render;
use super::super::db::RedisPool;

use schema::comment as comment_schema;
use schema::comment::table as comment_table;

use schema::ruser as ruser_schema;
use schema::ruser::table as ruser_table;

use std::sync::Arc;
use uuid::Uuid;
use chrono::NaiveDateTime;

use diesel;
use diesel::PgConnection;
use diesel::prelude::*;
use serde_json;


#[derive(Queryable, Debug, Clone, Deserialize, Serialize)]
pub struct CommentWithNickNameDto {
    id: Uuid,
    content: String,
    article_id: Uuid,
    author_id: Uuid,
    created_time: NaiveDateTime,
    status: i16, // 0 normal, 1 frozen, 2 deleted

    nickname: String,
}

impl CommentWithNickNameDto {
    pub fn query(
        conn: &PgConnection,
        limit: i64,
        offset: i64,
        article_id: Uuid,
    ) -> Result<Vec<Self>, String> {
        let res = comment_table
            .inner_join(ruser_table.on(comment_schema::author_id.eq(ruser::id)))
            .select((
                comment_schema::id,
                comment_schema::content,
                comment_schema::article_id,
                comment_schema::author_id,
                comment_schema::created_time,
                comment_schema::status,
                ruser::nickname,
            ))
            .filter(comment_schema::status.eq(0))
            .filter(comment_schema::article_id.eq(article_id))
            .order(comment_schema::created_time)
            .limit(limit)
            .offset(offset)
            .get_results::<CommentWithNickNameDto>(conn);

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
    ) -> Result<CommentsWithTotalDto<Self>, String> {
        let _res = comment_table
            .filter(comment_schema::article_id.eq(id))
            .filter(comment_schema::status.eq(0));

        let res = _res.inner_join(ruser_table.on(comment_schema::author_id.eq(ruser::id)))
            .select((
                comment_schema::id,
                comment_schema::content,
                comment_schema::article_id,
                comment_schema::author_id,
                comment_schema::created_time,
                comment_schema::status,
                ruser::nickname,
            ))
            .order(comment_schema::created_time)
            .offset(page_size * (page - 1) as i64)
            .limit(page_size)
            .get_results::<Self>(conn);

        let all_count: i64 = _res.count().get_result(conn).unwrap();

        match res {
            Ok(data) => Ok(CommentsWithTotalDto {
                comments: data,
                total: all_count,
                max_page: (all_count as f64 / page_size as f64).ceil() as i64,
            }),
            Err(err) => Err(format!("{}", err)),
        }
    }

    fn delete_with_comment_id(conn: &PgConnection, id: Uuid) -> bool {
        diesel::update(comment_table.filter(comment_schema::id.eq(id)))
            .set(comment_schema::status.eq(2))
            .execute(conn)
            .is_ok()
    }

    pub fn delete_with_author_id(conn: &PgConnection, id: Uuid) -> bool {
        diesel::update(comment_table.filter(comment_schema::author_id.eq(id)))
            .set(comment_schema::status.eq(2))
            .execute(conn)
            .is_ok()
    }
}

#[derive(Debug)]
pub struct CommentsWithTotalDto<T> {
    pub comments: Vec<T>,
    pub total: i64,
    pub max_page: i64,
}


#[derive(Insertable, Debug, Clone)]
#[table_name = "comment"]
struct InsertCommentDmo {
    content: String,
    article_id: Uuid,
    author_id: Uuid,
}

impl InsertCommentDmo {
    fn insert(self, conn: &PgConnection) -> bool {
        diesel::insert_into(comment_table)
            .values(&self)
            .execute(conn)
            .is_ok()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NewCommentDmo {
    pub content: String,
    pub article_id: Uuid,
    pub reply_user_id: Option<Uuid>,
}

impl NewCommentDmo {
    fn into_insert_comments(self, author_id: Uuid) -> InsertComment {
        InsertComment {
            content: markdown_render(&self.content),
            article_id: self.article_id,
            author_id: author_id,
        }
    }

    pub fn insert(self, conn: &PgConnection, redis_pool: &Arc<RedisPool>, cookie: &str) -> bool {
        let info =
            serde_json::from_str::<RUser>(&redis_pool.hget::<String>(cookie, "info")).unwrap();
        self.into_insert_comments(info.id).insert(conn)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DeleteCommentDmo {
    comment_id: Uuid,
    author_id: Uuid,
}

impl DeleteCommentDmo {
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
                let info = serde_json::from_str::<RUser>(&redis_pool
                    .hget::<String>(cookie, "info"))
                    .unwrap();
                if self.author_id == info.id {
                    CommentWithNickName::delete_with_comment_id(conn, self.comment_id)
                } else {
                    false
                }
            }
        }
    }
}
