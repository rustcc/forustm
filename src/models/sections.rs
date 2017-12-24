use super::super::section;
use super::super::section::dsl::section as all_sections;
use super::super::{RedisPool};
use sapper_std::Context;

use uuid::Uuid;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::PgConnection;
use diesel;

use std::sync::Arc;

#[derive(Queryable, Debug, Clone, Deserialize, Serialize)]
pub struct Section {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub stype: i32, // 0 section, 1 user blog
    pub suser: Option<Uuid>,
    pub created_time: NaiveDateTime,
    pub status: i16, // 0 normal, 1 frozen, 2 deleted
}

impl Section {
    pub fn query(conn: &PgConnection) -> Result<Vec<Self>, String> {
        let res = all_sections.filter(section::status.eq(0))
            .order(section::created_time.desc())
            .get_results::<Self>(conn);
        match res {
            Ok(data) => Ok(data),
            Err(err) => Err(format!("{}", err)),
        }
    }

    pub fn query_by_stype(conn: &PgConnection, stype: i32) -> Result<Vec<Self>, String> {
        let res = all_sections.filter(section::status.eq(0))
            .filter(section::stype.eq(stype))
            .order(section::created_time.desc())
            .get_results::<Self>(conn);
        match res {
            Ok(data) => Ok(data),
            Err(err) => Err(format!("{}", err)),
        }
    }

    pub fn query_with_section_id(conn: &PgConnection, id: Uuid) -> Result<Self, String> {
        let res = all_sections.filter(section::status.eq(0))
            .filter(section::id.eq(id))
            .first::<Self>(conn);
        match res {
            Ok(data) => {
                //println!("data {:?}", data);
                Ok(data)
            }
            Err(err) => Err(format!("{}", err)),
        }
    }

    pub fn query_with_user_id(conn: &PgConnection, id: Uuid) -> Result<Self, String> {
        let res = all_sections.filter(section::status.eq(0))
            .filter(section::suser.eq(id))
            .first::<Self>(conn);
        match res {
            Ok(data) => {
                //println!("data {:?}", data);
                Ok(data)
            }
            Err(err) => Err(format!("{}", err)),
        }
    }

    pub fn delete_with_id(conn: &PgConnection, id: Uuid) -> bool {
        diesel::update(all_sections.filter(section::id.eq(id)))
            .set(section::status.eq(2))
            .execute(conn)
            .is_ok()
    }
}

#[derive(Insertable, Debug, Clone, Deserialize, Serialize)]
#[table_name = "section"]
pub struct InsertSection {
    pub title: String,
    pub description: String,
    pub stype: i32,
    pub suser: Option<Uuid>,
}

impl InsertSection {
    pub fn insert(self, conn: &PgConnection) -> bool {
        diesel::insert_into(section::table)
            .values(&self)
            .execute(conn)
            .is_ok()
    }
}

// for redis
#[derive(Deserialize, Serialize)]
pub struct PubNotice {
    pub title: String,
    pub desc: String,
}

impl PubNotice {
    pub fn insert(self, redis_pool: &Arc<RedisPool> ) {
        redis_pool.hset("pub_notice", "title", self.title);
        redis_pool.hset("pub_notice", "desc", self.desc);
    }

    pub fn get(web: &mut Context, redis_pool: &Arc<RedisPool>) {
        if redis_pool.exists("pub_notice") {
            let title  = redis_pool.hget::<String>("pub_notice", "title");
            let desc  = redis_pool.hget::<String>("pub_notice", "desc");
            web.add("title", &title);
            web.add("desc", &desc);
        }

    }
}
