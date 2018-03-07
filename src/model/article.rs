use super::{RUserDto, Section}

use schema::article as article_schema;
use schema::article::table as article_table;

use schema::ruser as ruser_schema;
use schema::ruser::table as ruser_table;

use std::sync::Arc;
use chrono::NaiveDateTime;
use serde_json;
use uuid::Uuid;

use diesel;
use diesel::prelude::*;
use diesel::dsl::*;
use diesel::PgConnection;
use diesel::sql_types::BigInt;
use diesel::expression::SqlLiteral;

use super::super::util::markdown_render;
use super::super::db::RedisPool;


type SelectRawArticles = (
    article_schema::id,
    article_schema::title,
    article_schema::raw_content,
    article_schema::content,
    article_schema::section_id,
    article_schema::author_id,
    article_schema::tags,
    article_schema::stype,
    article_schema::created_time,
    article_schema::status,
    SqlLiteral<BigInt>,
    SqlLiteral<BigInt>,
);

fn select_raw_articles() -> SelectRawArticles {
    (
        article_schema::id,
        article_schema::title,
        article_schema::raw_content,
        article_schema::content,
        article_schema::section_id,
        article_schema::author_id,
        article_schema::tags,
        article_schema::stype,
        article_schema::created_time,
        article_schema::status,
        sql::<BigInt>(
            "(select (count(article_stats.id) + 1) from article_stats where article_stats.article_id = article.id)",
        ),
        sql::<BigInt>(
            "(select count(comment.id) from comment where comment.status = 0 and comment.article_id = article.id)",
        ),
    )
}

//
// MODEL
//

#[derive(Queryable)]
struct Article {
    id: Uuid,
    title: String,
    raw_content: String,
    content: String,
    section_id: Uuid,
    author_id: Uuid,
    tags: String,
    stype: i32, // 0 section, 1 user blog
    created_time: NaiveDateTime,
    status: i16, // 0 normal, 1 frozen, 2 deleted
    view_count: i64,
    comment_count: i64,
}

impl Article {
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
            stype: self.stype,

            view_count: self.view_count,
            comment_count: self.comment_count,
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
            stype: self.stype,

            view_count: self.view_count,
            comment_count: self.comment_count,
        }
    }

    // fn into_blog(self) -> Blog {
    //     Blog {
    //         id: self.id,
    //         title: self.title,
    //         author_id: self.author_id,
    //         section_id: self.section_id,
    //         tags: self.tags,
    //         content: self.content,
    //         created_time: self.created_time,
    //     }
    // }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ArticleDto {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub section_id: Uuid,
    pub author_id: Uuid,
    pub tags: String,
    pub created_time: NaiveDateTime,
    pub status: i16,
    pub stype: i32,

    pub view_count: i64,
    pub comment_count: i64,
}

impl ArticleDto {
    pub fn query_article(conn: &PgConnection, id: Uuid) -> Result<ArticleDto, String> {
        let res = article_table
            .select(select_raw_articles())
            .filter(article_schema::status.ne(2))
            .filter(article_schema::id.eq(id))
            .get_result::<Article>(conn);
        
        match res {
            Ok(data) => Ok(data.into_html()),
            Err(err) => Err(format!("{}", err)),
        }
    }

    pub fn query_article_md(conn: &PgConnection, id: Uuid) -> Result<ArticleDto, String> {
        let res = article_table
            .filter(article_schema::status.ne(2))
            .filter(article_schema::id.eq(id))
            .select(select_raw_articles())
            .get_result::<Article>(conn);

        match res {
            Ok(data) => Ok(data.into_markdown()),
            Err(err) => Err(format!("{}", err)),
        }
    }

    pub fn query_blogs(conn: &PgConnection, id: Uuid) -> Result<ArticleDto, String> {
        let res = article_table
            .filter(article_schema::status.ne(2))
            .filter(article_schema::id.eq(id))
            .filter(article_schema::stype.eq(1))
            .select(select_raw_articles())
            .get_result::<Article>(conn);

        match res {
            Ok(data) => Ok(data.into_html()),
            Err(err) => Err(format!("{}", err)),
        }
    }

    pub fn query_raw_article(conn: &PgConnection, id: Uuid) -> Result<ArticleDto, String> {
        let res = article_table
            .filter(article_schema::status.ne(2))
            .filter(article_schema::id.eq(id))
            .select(select_raw_articles())
            .get_result::<Article>(conn);
        match res {
            Ok(data) => Ok(data.into_markdown()),
            Err(err) => Err(format!("{}", err)),
        }
    }

    fn raw_articles_with_section_id(
        conn: &PgConnection,
        id: Uuid,
    ) -> Result<Vec<RawArticles>, String> {
        let res = article_table
            .filter(article_schema::section_id.eq(id))
            .filter(article_schema::status.ne(2))
            .select(select_raw_articles())
            .order(article_schema::created_time.desc())
            .get_results::<RawArticles>(conn);
        match res {
            Ok(data) => Ok(data),
            Err(err) => Err(format!("{}", err)),
        }
    }

    pub fn query_articles_with_section_id(
        conn: &PgConnection,
        id: Uuid,
    ) -> Result<Vec<Article>, String> {
        match Article::raw_articles_with_section_id(conn, id) {
            Ok(raw_articles) => Ok(raw_articles
                .into_iter()
                .map(|art| art.into_html())
                .collect::<Vec<Article>>()),
            Err(err) => Err(err),
        }
    }

    #[allow(warnings)]
    fn raw_articles_with_section_id_paging(
        conn: &PgConnection,
        id: Uuid,
        page: i64,
        page_size: i64,
    ) -> Result<ArticlesWithTotal<RawArticles>, String> {
        let _res = article_table
            .filter(article_schema::section_id.eq(id))
            .filter(article_schema::status.ne(2));

        let res = _res.select(select_raw_articles())
            .order(article_schema::created_time.desc())
            .offset(page_size * (page - 1) as i64)
            .limit(page_size)
            .get_results::<RawArticles>(conn);

        let all_count: i64 = _res.count().get_result(conn).unwrap();

        match res {
            Ok(data) => Ok(ArticlesWithTotal {
                articles: data,
                total: all_count,
                max_page: (all_count as f64 / page_size as f64).ceil() as i64,
            }),
            Err(err) => Err(format!("{}", err)),
        }
    }

    pub fn query_articles_with_section_id_paging(
        conn: &PgConnection,
        id: Uuid,
        page: i64,
        page_size: i64,
    ) -> Result<ArticlesWithTotal<ArticleBrief>, String> {
        let _res = article_table
            .filter(article_schema::section_id.eq(id))
            .filter(article_schema::status.ne(2));

        let res = _res.inner_join(ruser_table.on(article_schema::author_id.eq(ruser::id)))
            .select((
                article_schema::id,
                article_schema::title,
                article_schema::author_id,
                article_schema::tags,
                article_schema::created_time,
                ruser::nickname,
                sql::<BigInt>("(select count(article_stats.id) from article_stats where article_stats.article_id = article.id)"),
                sql::<BigInt>("(select count(comment.id) from comment where comment.status = 0 and comment.article_id = article.id)"),
            ))
            .order(article_schema::created_time.desc())
            .offset(page_size * (page - 1) as i64)
            .limit(page_size)
            .get_results::<ArticleBrief>(conn);

        let all_count: i64 = _res.count().get_result(conn).unwrap();

        match res {
            Ok(data) => Ok(ArticlesWithTotal {
                articles: data,
                total: all_count,
                max_page: (all_count as f64 / page_size as f64).ceil() as i64,
            }),
            Err(err) => Err(format!("{}", err)),
        }
    }

    pub fn query_articles_with_section_id_and_stype_paging(
        conn: &PgConnection,
        id: Uuid,
        stype: i32,
        page: i64,
        page_size: i64,
    ) -> Result<ArticlesWithTotal<ArticleBrief>, String> {
        let _res = article_table
            .filter(article_schema::section_id.eq(id))
            .filter(article_schema::stype.eq(stype))
            .filter(article_schema::status.ne(2));

        let res = _res.inner_join(ruser_table.on(article_schema::author_id.eq(ruser::id)))
            .select((
                article_schema::id,
                article_schema::title,
                article_schema::author_id,
                article_schema::tags,
                article_schema::created_time,
                ruser::nickname,
                sql::<BigInt>("(select count(article_stats.id) from article_stats where article_stats.article_id = article.id)"),
                sql::<BigInt>("(select count(comment.id) from comment where comment.status = 0 and comment.article_id = article.id)"),
            ))
            .order(article_schema::created_time.desc())
            .offset(page_size * (page - 1) as i64)
            .limit(page_size)
            .get_results::<ArticleBrief>(conn);

        let all_count: i64 = _res.count().get_result(conn).unwrap();

        match res {
            Ok(data) => Ok(ArticlesWithTotal {
                articles: data,
                total: all_count,
                max_page: (all_count as f64 / page_size as f64).ceil() as i64,
            }),
            Err(err) => Err(format!("{}", err)),
        }
    }

    // fn raw_articles_by_stype_paging(conn: &PgConnection,
    //                                 stype: i32,
    //                                 page: i64,
    //                                 page_size: i64)
    //                                 -> Result<ArticlesWithTotal<RawArticles>, String> {
    //     let _res = article_table.filter(article_schema::stype.eq(stype))
    //         .filter(article_schema::status.ne(2));

    //     let res = _res.order(article_schema::created_time.desc())
    //         .offset(page_size * (page - 1) as i64)
    //         .limit(page_size)
    //         .get_results::<RawArticles>(conn);

    //     let all_count: i64 = _res.count()
    //         .get_result(conn)
    //         .unwrap();

    //     match res {
    //         Ok(data) => {
    //             Ok(ArticlesWithTotal {
    //                 articles: data,
    //                 total: all_count,
    //                 max_page: (all_count as f64 / page_size as f64).ceil() as i64,
    //             })
    //         }
    //         Err(err) => Err(format!("{}", err)),
    //     }
    // }

    pub fn query_blogs_paging(
        conn: &PgConnection,
        stype: i32,
        page: i64,
        page_size: i64,
    ) -> Result<ArticlesWithTotal<BlogBrief>, String> {
        let _res = article_table
            .filter(article_schema::stype.eq(stype))
            .filter(article_schema::status.ne(2));

        let res = _res.inner_join(ruser_table.on(article_schema::author_id.eq(ruser::id)))
            .select((
                article_schema::id,
                article_schema::title,
                article_schema::author_id,
                article_schema::tags,
                article_schema::created_time,
                ruser::nickname,
                sql::<BigInt>("(select count(article_stats.id) from article_stats where article_stats.article_id = article.id)"),
                sql::<BigInt>("(select count(comment.id) from comment where comment.status = 0 and comment.article_id = article.id)"),
            ))
            .order(article_schema::created_time.desc())
            .offset(page_size * (page - 1) as i64)
            .limit(page_size)
            .get_results::<BlogBrief>(conn);

        let all_count: i64 = _res.count().get_result(conn).unwrap();

        match res {
            Ok(data) => Ok(ArticlesWithTotal {
                articles: data,
                total: all_count,
                max_page: (all_count as f64 / page_size as f64).ceil() as i64,
            }),
            Err(err) => Err(format!("{}", err)),
        }
    }

    pub fn delete_with_id(conn: &PgConnection, id: Uuid) -> Result<usize, String> {
        let res = diesel::update(article_table.filter(article_schema::id.eq(id)))
            .set(article_schema::status.eq(2))
            .execute(conn);
        match res {
            Ok(data) => Ok(data),
            Err(err) => Err(format!("{}", err)),
        }
    }
}


#[derive(Queryable, Debug, Clone, Deserialize, Serialize)]
pub struct ArticleBriefDto {
    pub id: Uuid,
    pub title: String,
    pub author_id: Uuid,
    pub tags: String,
    pub created_time: NaiveDateTime,

    pub author_name: String,
    pub view_count: i64,
    pub comment_count: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BlogDto {
    pub id: Uuid,
    pub title: String,
    pub author_id: Uuid,
    pub section_id: Uuid,
    pub tags: String,
    pub content: String,
    pub created_time: NaiveDateTime,
}

#[derive(Queryable, Debug, Clone, Deserialize, Serialize)]
pub struct BlogBriefDto {
    pub id: Uuid,
    pub title: String,
    pub author_id: Uuid,
    pub tags: String,
    pub created_time: NaiveDateTime,
    pub author_name: String,

    pub view_count: i64,
    pub comment_count: i64,
}

#[derive(Debug)]
pub struct ArticlesWithTotalDto<T> {
    pub articles: Vec<T>,
    pub total: i64,
    pub max_page: i64,
}


#[derive(Queryable, Debug, Clone, Deserialize, Serialize)]
pub struct SimpleArticleDto {
    pub id: Uuid,
    pub title: String,
    pub author_id: Uuid,
}

impl SimpleArticleDto {
    pub fn query_simple_article(conn: &PgConnection, id: Uuid) -> Result<SimpleArticle, String> {
        let res = article_table
            .filter(article_schema::status.ne(2))
            .filter(article_schema::id.eq(id))
            .select((article_schema::id, article_schema::title, article_schema::author_id))
            .get_result::<SimpleArticleDto>(conn);
        match res {
            Ok(data) => Ok(data),
            Err(err) => Err(format!("{}", err)),
        }
    }
}

#[derive(Insertable, Debug, Clone)]
#[table_name = "article"]
struct InsertArticleDmo {
    title: String,
    raw_content: String,
    content: String,
    section_id: Uuid,
    author_id: Uuid,
    stype: i32,
    tags: String,
}

impl InsertArticleDmo {
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
        let res = diesel::insert_into(article_table)
            .values(&self)
            .execute(conn);
        match res {
            Ok(data) => Ok(data),
            Err(err) => Err(format!("{}", err)),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct NewArticleDmo {
    pub title: String,
    pub raw_content: String,
    pub section_id: Uuid,
    pub stype: i32,
    pub tags: String,
}

impl NewArticleDmo {
    pub fn insert(
        self,
        conn: &PgConnection,
        redis_pool: &Arc<RedisPool>,
        cookie: &str,
    ) -> Result<usize, String> {
        let user: RUser =
            serde_json::from_str(&RUser::view_with_cookie(redis_pool, cookie)).unwrap();
        if self.stype == 1 {
            let blog_owner = Section::query_with_section_id(conn, self.section_id)
                .unwrap()
                .suser
                .unwrap();
            if user.id == blog_owner {
                InsertArticle::new(self, user.id).insert(conn)
            } else {
                Err("No right to add articles".to_string())
            }
        } else {
            InsertArticle::new(self, user.id).insert(conn)
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct EditArticleDmo {
    id: Uuid,
    title: String,
    raw_content: String,
    tags: String,
    author_id: Uuid,
}

impl EditArticleDmo {
    pub fn edit_article(
        self,
        conn: &PgConnection,
        redis_pool: &Arc<RedisPool>,
        cookie: &str,
    ) -> Result<usize, String> {
        let info =
            serde_json::from_str::<RUser>(&redis_pool.hget::<String>(cookie, "info")).unwrap();
        if self.author_id == info.id {
            let res = diesel::update(article_table.filter(article_schema::id.eq(self.id)))
                .set((
                    article_schema::title.eq(self.title),
                    article_schema::content.eq(markdown_render(&self.raw_content)),
                    article_schema::raw_content.eq(self.raw_content),
                    article_schema::tags.eq(self.tags),
                ))
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
pub struct DeleteArticleDmo {
    article_id: Uuid,
    user_id: Uuid,
}

impl DeleteArticleDmo {
    pub fn delete(
        self,
        conn: &PgConnection,
        redis_pool: &Arc<RedisPool>,
        cookie: &str,
        permission: &Option<i16>,
    ) -> bool {
        match *permission {
            Some(0) | Some(1) => Article::delete_with_id(conn, self.article_id).is_ok(),
            _ => {
                let logged_user = serde_json::from_str::<RUser>(&redis_pool
                    .hget::<String>(cookie, "info"))
                    .unwrap();
                if self.user_id == logged_user.id {
                    Article::delete_with_id(conn, self.article_id).is_ok()
                } else {
                    false
                }
            }
        }
    }
}


#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChangStatusDmo {
    pub id: Uuid,
    pub status: i16,
}
