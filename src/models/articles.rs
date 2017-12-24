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
    #[allow(warnings)]
    stype: i32, // 0 section, 1 user blog
    created_time: NaiveDateTime,
    status: i16, // 0 normal, 1 frozen, 2 deleted
}

impl RawArticles {
    fn into_html(self) -> Article {
        Article {
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

    fn into_markdown(self) -> Article {
        Article {
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

    fn into_brief(self) -> ArticleBrief {
        ArticleBrief {
            id: self.id,
            title: self.title,
            author_id: self.author_id,
            tags: self.tags,
            created_time: self.created_time,
        }
    }

    fn into_blog(self) -> Blog {
        Blog {
            id: self.id,
            title: self.title,
            author_id: self.author_id,
            section_id: self.section_id,
            tags: self.tags,
            content: self.content,
            created_time: self.created_time,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Article {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub section_id: Uuid,
    pub author_id: Uuid,
    pub tags: String,
    pub created_time: NaiveDateTime,
    pub status: i16,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ArticleBrief {
    pub id: Uuid,
    pub title: String,
    pub author_id: Uuid,
    pub tags: String,
    pub created_time: NaiveDateTime,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Blog {
    pub id: Uuid,
    pub title: String,
    pub author_id: Uuid,
    pub section_id: Uuid,
    pub tags: String,
    pub content: String,
    pub created_time: NaiveDateTime,
}

#[derive(Debug)]
pub struct ArticlesWithTotal<T> {
    pub articles: Vec<T>,
    pub total: i64,
    pub max_page: i64,
}



impl Article {
    pub fn query_article(conn: &PgConnection, id: Uuid) -> Result<Article, String> {
        let res = all_articles.filter(article::status.ne(2))
            .filter(article::id.eq(id))
            .get_result::<RawArticles>(conn);
        match res {
            Ok(data) => Ok(data.into_html()),
            Err(err) => Err(format!("{}", err)),
        }
    }

    pub fn query_article_md(conn: &PgConnection, id: Uuid) -> Result<Article, String> {
        let res = all_articles.filter(article::status.ne(2))
            .filter(article::id.eq(id))
            .get_result::<RawArticles>(conn);
        match res {
            Ok(data) => Ok(data.into_markdown()),
            Err(err) => Err(format!("{}", err)),
        }
    }

    pub fn query_blogs(conn: &PgConnection, id: Uuid) -> Result<Article, String> {
        let res = all_articles.filter(article::status.ne(2))
            .filter(article::id.eq(id))
            .filter(article::stype.eq(1))
            .get_result::<RawArticles>(conn);
        match res {
            Ok(data) => Ok(data.into_html()),
            Err(err) => Err(format!("{}", err)),
        }
    }

    pub fn query_raw_article(conn: &PgConnection, id: Uuid) -> Result<Article, String> {
        let res = all_articles.filter(article::status.ne(2))
            .filter(article::id.eq(id))
            .get_result::<RawArticles>(conn);
        match res {
            Ok(data) => Ok(data.into_markdown()),
            Err(err) => Err(format!("{}", err)),
        }
    }

    fn raw_articles_with_section_id(conn: &PgConnection, id: Uuid) -> Result<Vec<RawArticles>, String> {
        let res = all_articles
            .filter(article::section_id.eq(id))
            .filter(article::status.ne(2))
            .order(article::created_time.desc())
            .get_results::<RawArticles>(conn);
        match res {
            Ok(data) => {
                Ok(data)
            }
            Err(err) => Err(format!("{}", err)),
        }
    }

    pub fn query_articles_with_section_id(conn: &PgConnection, id: Uuid)
                                          -> Result<Vec<Article>, String> {
        match Article::raw_articles_with_section_id(conn, id) {
            Ok(raw_articles) => {
                Ok(raw_articles.into_iter()
                    .map(|art| art.into_html())
                    .collect::<Vec<Article>>())
            }
            Err(err) => Err(err)
        }
    }


    fn raw_articles_with_section_id_paging(conn: &PgConnection, id: Uuid, page: i64, page_size: i64)
            -> Result<ArticlesWithTotal<RawArticles>, String> {
        let _res = all_articles
            .filter(article::section_id.eq(id))
            .filter(article::status.ne(2));

        let res = _res
            .order(article::created_time.desc())
            .offset(page_size * (page - 1) as i64)
            .limit(page_size)
            .get_results::<RawArticles>(conn);

        let all_count: i64 = _res
            .count()
            .get_result(conn).unwrap();

        match res {
            Ok(data) => {
                Ok(ArticlesWithTotal {
                    articles: data,
                    total: all_count,
                    max_page: (all_count as f64 / page_size as f64).ceil() as i64,
                })
            }
            Err(err) => Err(format!("{}", err)),
        }
    }

    pub fn query_articles_with_section_id_paging(conn: &PgConnection, id: Uuid, page: i64, page_size: i64)
          -> Result<ArticlesWithTotal<ArticleBrief>, String> {
        match Article::raw_articles_with_section_id_paging(conn, id, page, page_size) {
            Ok(raw_articles) => {
                Ok(
                    ArticlesWithTotal{
                        articles: raw_articles.articles.into_iter()
                            .map(|art| art.into_brief())
                            .collect::<Vec<ArticleBrief>>(),
                        total: raw_articles.total,
                        max_page: raw_articles.max_page,
                    }
                )
            }
            Err(err) => Err(err)
        }
    }

    fn raw_articles_by_stype_paging(conn: &PgConnection, stype: i32, page: i64, page_size: i64)
            -> Result<ArticlesWithTotal<RawArticles>, String> {
        let _res = all_articles
            .filter(article::stype.eq(stype))
            .filter(article::status.ne(2));

        let res = _res
            .order(article::created_time.desc())
            .offset(page_size * (page - 1) as i64)
            .limit(page_size)
            .get_results::<RawArticles>(conn);

        let all_count: i64 = _res
            .count()
            .get_result(conn).unwrap();

        match res {
            Ok(data) => {
                Ok(ArticlesWithTotal {
                    articles: data,
                    total: all_count,
                    max_page: (all_count as f64 / page_size as f64).ceil() as i64,
                })
            }
            Err(err) => Err(format!("{}", err)),
        }
    }

    pub fn query_articles_by_stype_paging(conn: &PgConnection, stype: i32, page: i64, page_size: i64)
          -> Result<ArticlesWithTotal<Blog>, String> {
        match Article::raw_articles_by_stype_paging(conn, stype, page, page_size) {
            Ok(raw_articles) => {
                Ok(
                    ArticlesWithTotal{
                        articles: raw_articles.articles.into_iter()
                            .map(|art| art.into_blog())
                            .collect::<Vec<Blog>>(),
                        total: raw_articles.total,
                        max_page: raw_articles.max_page,
                    }
                )
            }
            Err(err) => Err(err)
        }
    }

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
    stype: i32,
    tags: String,
}

impl InsertArticle {
    fn new(new_article: NewArticle, author_id: Uuid) -> Self {
        let content = markdown_render(&new_article.raw_content);
        InsertArticle {
            title: new_article.title,
            raw_content: new_article.raw_content,
            content: content,
            section_id: new_article.section_id,
            author_id: author_id,
            stype: new_article.stype,
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
    pub stype: i32,
    pub tags: String,
}

impl NewArticle {
    pub fn insert(self, conn: &PgConnection, redis_pool: &Arc<RedisPool>, cookie: &str) -> Result<usize, String>  {
        let user:RUser = serde_json::from_str(&RUser::view_with_cookie(redis_pool, cookie)).unwrap();
        InsertArticle::new(self, user.id).insert(conn)
    }
}

#[derive(Deserialize, Serialize)]
pub struct EditArticle {
    id: Uuid,
    title: String,
    raw_content: String,
    tags: String,
    author_id: Uuid,
}

impl EditArticle {
    pub fn edit_article(self, conn: &PgConnection, redis_pool: &Arc<RedisPool>, cookie: &str) -> Result<usize, String> {
        let info =
            serde_json::from_str::<RUser>(&redis_pool.hget::<String>(cookie, "info"))
                .unwrap();
        if self.author_id == info.id {
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
        } else {
            Err("No permission".to_string())
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
            &Some(0) | &Some(1) => Article::delete_with_id(conn, self.article_id).is_ok(),
            _ => {
                let logged_user =
                    serde_json::from_str::<RUser>(&redis_pool.hget::<String>(cookie, "info"))
                        .unwrap();
                match self.user_id == logged_user.id {
                    true => Article::delete_with_id(conn, self.article_id).is_ok(),
                    false => false,
                }
            }
        }
    }
}
