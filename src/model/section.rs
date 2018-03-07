use super::super::db::RedisPool;
use schema::section as section_schema;
use schema::section::table as section_table;

use std::sync::Arc;
use sapper_std::Context;
use uuid::Uuid;
use chrono::NaiveDateTime;

use diesel;
use diesel::PgConnection;
use diesel::pg::expression::dsl::any;
use diesel::prelude::*;


//
// MODEL
//

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
        let res = section_table
            .filter(section_schema::status.eq(0))
            .order(section_schema::created_time.desc())
            .get_results::<Self>(conn);
        match res {
            Ok(data) => Ok(data),
            Err(err) => Err(format!("{}", err)),
        }
    }

    pub fn query_by_stype(conn: &PgConnection, stype: i32) -> Result<Vec<Self>, String> {
        let res = section_table
            .filter(section_schema::status.eq(0))
            .filter(section_schema::stype.eq(stype))
            .order(section_schema::created_time.desc())
            .get_results::<Self>(conn);
        match res {
            Ok(data) => Ok(data),
            Err(err) => Err(format!("{}", err)),
        }
    }

    pub fn query_with_section_id(conn: &PgConnection, id: Uuid) -> Result<Self, String> {
        let res = section_table
            .filter(section_schema::status.eq(0))
            .filter(section_schema::id.eq(id))
            .first::<Self>(conn);
        match res {
            Ok(data) => {
                // println!("data {:?}", data);
                Ok(data)
            }
            Err(err) => Err(format!("{}", err)),
        }
    }

    pub fn query_with_user_id(conn: &PgConnection, id: Uuid) -> Result<Self, String> {
        let res = section_table
            .filter(section_schema::status.eq(0))
            .filter(section_schema::suser.eq(id))
            .first::<Self>(conn);
        match res {
            Ok(data) => {
                // println!("data {:?}", data);
                Ok(data)
            }
            Err(err) => Err(format!("{}", err)),
        }
    }

    pub fn delete_with_id(conn: &PgConnection, id: Uuid) -> bool {
        diesel::update(section_table.filter(section_schema::id.eq(id)))
            .set(section_schema::status.eq(2))
            .execute(conn)
            .is_ok()
    }

    pub fn query_with_redis_queue(
        conn: &PgConnection,
        redis_pool: &Arc<RedisPool>,
        key: &'static str,
    ) -> Result<Vec<Self>, String> {
        if redis_pool.exists(key) {
            let section_ids_string = redis_pool.lrange::<Vec<String>>(key, 0, -1);
            let section_ids: Vec<Uuid> = section_ids_string
                .into_iter()
                .map(|id_str| id_str.parse::<Uuid>().unwrap())
                .collect();

            let res = section_table
                .filter(section_schema::status.eq(0))
                .filter(section_schema::id.eq(any(section_ids)))
                .get_results::<Self>(conn);
            match res {
                Ok(data) => Ok(data),
                Err(err) => Err(format!("{}", err)),
            }
        } else {
            Ok(vec![])
        }
    }
}

#[derive(Insertable, Debug, Clone, Deserialize, Serialize)]
#[table_name = "section"]
pub struct InsertSectionDmo {
    pub title: String,
    pub description: String,
    pub stype: i32,
    pub suser: Option<Uuid>,
}

impl InsertSectionDmo {
    pub fn insert(self, conn: &PgConnection) -> bool {
        diesel::insert_into(section_table)
            .values(&self)
            .execute(conn)
            .is_ok()
    }
}

//
// DTOs
//


// for redis
#[derive(Deserialize, Serialize)]
pub struct PubNotice {
    pub title: String,
    pub desc: String,
}

impl PubNotice {
    pub fn insert(self, redis_pool: &Arc<RedisPool>) {
        redis_pool.hset("pub_notice", "title", self.title);
        redis_pool.hset("pub_notice", "desc", self.desc);
    }

    pub fn get(web: &mut Context, redis_pool: &Arc<RedisPool>) {
        if redis_pool.exists("pub_notice") {
            let title = redis_pool.hget::<String>("pub_notice", "title");
            let desc = redis_pool.hget::<String>("pub_notice", "desc");
            web.add("title", &title);
            web.add("desc", &desc);
        }
    }
}
