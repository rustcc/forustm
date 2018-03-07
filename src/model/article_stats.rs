use schema::article_stats as article_stats_schema;
use schema::article_stats::table as article_stats_table;

use chrono::NaiveDateTime;
use uuid::Uuid;

use diesel;
use diesel::PgConnection;
use diesel::prelude::*;


#[derive(Queryable, Debug, Clone, Serialize, Deserialize)]
pub struct ArticleStats {
    pub id: Uuid,
    pub article_id: Uuid,
    pub created_time: NaiveDateTime,
    pub ruser_id: Option<Uuid>,
    pub user_agent: Option<String>,
    pub visitor_ip: Option<String>,
}

#[derive(Insertable, Debug, Clone)]
#[table_name = "article_stats"]
pub struct NewArticleStatsDmo {
    pub article_id: Uuid,
    pub ruser_id: Option<Uuid>,
    pub user_agent: Option<String>,
    pub visitor_ip: Option<String>,
}

impl NewArticleStatsDmo {
    pub fn insert(self, conn: &PgConnection) -> Result<usize, String> {
        let res = diesel::insert_into(article_stats_table)
            .values(&self)
            .execute(conn);
        match res {
            Ok(data) => Ok(data),
            Err(err) => Err(format!("{}", err)),
        }
    }
}
