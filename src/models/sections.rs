use super::super::section;
use super::super::section::dsl::section as all_sections;

use uuid::Uuid;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::PgConnection;
use diesel;


#[derive(Queryable, Debug, Clone, Deserialize, Serialize)]
pub struct Section {
    id: Uuid,
    title: String,
    description: String,
    stype: i32,
    suser: Uuid,
    created_time: NaiveDateTime
}

impl Section {
    pub fn query(conn: &PgConnection) -> Result<Vec<Self>, String> {
        let res = all_sections
            .order(section::created_time.desc())
            .get_results::<Self>(conn);
        match res {
            Ok(data) => {
                Ok(data)
            },
            Err(err) => Err(format!("{}", err))
        }
    }

    pub fn delete_with_id(conn: &PgConnection, id: Uuid) -> bool {
        diesel::delete(all_sections.filter(section::id.eq(id)))
            .execute(conn).is_ok()
    }
}

#[derive(Insertable, Debug, Clone, Deserialize, Serialize)]
#[table_name = "section"]
pub struct InsertSection {
    pub title: String,
    pub description: String,
    pub stype: i32,
    pub suser: Uuid,
}

impl InsertSection {
    pub fn insert(self, conn: &PgConnection) -> bool {
        diesel::insert_into(section::table)
            .values(&self)
            .execute(conn).is_ok()
    }
}

