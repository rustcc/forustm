use super::super::article::dsl::article as all_articles;
use super::super::article;
use super::super::{markdown_render, RUser, RedisPool};

use chrono::NaiveDateTime;
use uuid::Uuid;
use diesel;
use diesel::prelude::*;
use diesel::PgConnection;
use std::sync::Arc;
use serde_json;


#[derive(Queryable)]
struct RawArticles {
    id: Uuid,
    title: String,
    raw_content: String,
    content: String,
    section_id: Uuid,
    author_id: Uuid,
    tags: String,
    created_time: NaiveDateTime,
    status: i16, // 0 normal, 1 frozen, 2 deleted
}

impl RawArticles {
    fn into_html(self) -> Articles {
        Articles {
            id: self.id,
            title: self.title,
            content: self.content,
            section_id: self.section_id,
            author_id: self.author_id,
            tags: self.tags,
            created_time: self.created_time,
            status: self.status,
        }
    }

    fn into_markdown(self) -> Articles {
        Articles {
            id: self.id,
            title: self.title,
            content: self.raw_content,
            section_id: self.section_id,
            author_id: self.author_id,
            tags: self.tags,
            created_time: self.created_time,
            status: self.status,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Articles {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub section_id: Uuid,
    pub author_id: Uuid,
    pub tags: String,
    pub created_time: NaiveDateTime,
    pub status: i16,
}

impl Articles {
    pub fn query_article(conn: &PgConnection, id: Uuid) -> Result<Articles, String> {
        let res = all_articles.filter(article::status.eq(0))
            .filter(article::id.eq(id))
            .get_result::<RawArticles>(conn);
        match res {
            Ok(data) => Ok(data.into_html()),
            Err(err) => Err(format!("{}", err)),
        }
    }

    pub fn query_raw_article(conn: &PgConnection, id: Uuid) -> Result<Articles, String> {
        let res = all_articles.filter(article::status.eq(0))
            .filter(article::id.eq(id))
            .get_result::<RawArticles>(conn);
        match res {
            Ok(data) => Ok(data.into_markdown()),
            Err(err) => Err(format!("{}", err)),
        }
    }

    pub fn query_articles_with_section_id(conn: &PgConnection,
                                          id: Uuid)
                                          -> Result<Vec<Articles>, String> {
        let res = all_articles.filter(article::status.eq(0))
            .filter(article::section_id.eq(id))
            .get_results::<RawArticles>(conn);
        match res {
            Ok(data) => {
                Ok(data.into_iter()
                    .map(|art| art.into_html())
                    .collect::<Vec<Articles>>())
            }
            Err(err) => Err(format!("{}", err)),
        }
    }

    /*
    pub fn query_raw_articles_with_section_id(conn: &PgConnection, id: Uuid) -> Result<Vec<Articles>, String> {
    }
    */

    pub fn delete_with_id(conn: &PgConnection, id: Uuid) -> Result<usize, String> {
        let res = diesel::update(all_articles.filter(article::id.eq(id)))
            .set(article::status.eq(2))
            .execute(conn);
        match res {
            Ok(data) => Ok(data),
            Err(err) => Err(format!("{}", err)),
        }
    }
}

#[derive(Insertable, Debug, Clone)]
#[table_name = "article"]
struct InsertArticle {
    title: String,
    raw_content: String,
    content: String,
    section_id: Uuid,
    author_id: Uuid,
    tags: String,
}

impl InsertArticle {
    fn new(new_article: NewArticle) -> Self {
        let content = markdown_render(&new_article.raw_content);
        InsertArticle {
            title: new_article.title,
            raw_content: new_article.raw_content,
            content: content,
            section_id: new_article.section_id,
            author_id: new_article.author_id,
            tags: new_article.tags,
        }
    }

    fn insert(self, conn: &PgConnection) -> Result<usize, String> {
        let res = diesel::insert_into(all_articles)
            .values(&self)
            .execute(conn);
        match res {
            Ok(data) => Ok(data),
            Err(err) => Err(format!("{}", err)),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct NewArticle {
    pub title: String,
    pub raw_content: String,
    pub section_id: Uuid,
    pub author_id: Uuid,
    pub tags: String,
}

impl NewArticle {
    pub fn insert(self, conn: &PgConnection) -> bool {
        InsertArticle::new(self).insert(conn).is_ok()
    }
}

#[derive(Deserialize, Serialize)]
pub struct EditArticle {
    id: Uuid,
    title: String,
    raw_content: String,
    tags: String,
}

impl EditArticle {
    pub fn edit_article(self, conn: &PgConnection) -> Result<usize, String> {
        let res = diesel::update(all_articles.filter(article::id.eq(self.id)))
            .set((article::title.eq(self.title),
                  article::content.eq(markdown_render(&self.raw_content)),
                  article::raw_content.eq(self.raw_content),
                  article::tags.eq(self.tags)))
            .execute(conn);
        match res {
            Ok(data) => Ok(data),
            Err(err) => Err(format!("{}", err)),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct DeleteArticle {
    article_id: Uuid,
    user_id: Uuid,
}

impl DeleteArticle {
    pub fn delete(self,
                  conn: &PgConnection,
                  redis_pool: &Arc<RedisPool>,
                  cookie: &str,
                  permission: &Option<i16>)
                  -> bool {
        match permission {
            &Some(0) | &Some(1) => Articles::delete_with_id(conn, self.article_id).is_ok(),
            _ => {
                let info =
                    serde_json::from_str::<RUser>(&redis_pool.hget::<String>(cookie, "info"))
                        .unwrap();
                match self.user_id == info.id {
                    true => Articles::delete_with_id(conn, self.article_id).is_ok(),
                    false => false,
                }
            }
        }
    }
}
