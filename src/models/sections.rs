use super::super::section;
use super::super::section::dsl::section as all_sections;

use uuid::Uuid;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::PgConnection;
use diesel;


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

    pub fn query_with_id(conn: &PgConnection, id: Uuid) -> Result<Self, String> {
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
