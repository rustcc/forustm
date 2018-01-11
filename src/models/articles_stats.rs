use super::super::{article_stats};
use super::super::article_stats::dsl::article_stats as all_articles_stats;

use chrono::NaiveDateTime;
use diesel;
use diesel::PgConnection;
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Queryable, Debug, Clone, Serialize, Deserialize)]
pub struct QueryArticleStats {
    pub id: Uuid,
    pub article_id: Uuid,
    pub created_time: NaiveDateTime,
    pub ruser_id: Option<Uuid>,
    pub user_agent: Option<String>,
    pub visitor_ip: Option<String>,
}

#[derive(Insertable, Debug, Clone)]
#[table_name = "article_stats"]
pub struct NewArticleStats {
    pub article_id: Uuid,
    pub ruser_id: Option<Uuid>,
    pub user_agent: Option<String>,
    pub visitor_ip: Option<String>,
}

impl NewArticleStats {
    pub fn insert(self, conn: &PgConnection) -> Result<usize, String> {
        let res = diesel::insert_into(all_articles_stats)
            .values(&self)
            .execute(conn);
        match res {
            Ok(data) => Ok(data),
            Err(err) => Err(format!("{}", err)),
        }
    }
}