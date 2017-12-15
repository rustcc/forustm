use super::super::ruser;
use super::super::ruser::dsl::ruser as all_rusers;
use super::super::{ sha3_256_encode, random_string, RedisPool, InsertSection };

use uuid::Uuid;
use chrono::{ NaiveDateTime, Local };
use diesel;
use diesel::prelude::*;
use diesel::PgConnection;
use serde_json;
use std::sync::Arc;


#[derive(Queryable)]
struct RawUser {
    pub id: Uuid,
    // email
    pub account: String,
    pub password: String,
    pub salt: String,
    pub nickname: String,
    pub avatar: Option<String>,
    pub wx_openid: Option<String>,
    pub say: Option<String>,
    pub signup_time: NaiveDateTime,
    pub role: i16, // member => 2, manager => 1, admin => 0
    pub status: i16 // 0 normal, 1 frozen, 2 deleted
}

impl RawUser {
    fn into_user_info(self) -> RUser {
        RUser {
            id: self.id,
            account: self.account,
            nickname: self.nickname,
            say: self.say,
            avatar: self.avatar,
            wx_openid: self.wx_openid,
            signup_time: self.signup_time,
            role: self.role,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RUser {
    pub id: Uuid,
    pub account: String,
    pub nickname: String,
    pub say: Option<String>,
    pub avatar: Option<String>,
    pub wx_openid: Option<String>,
    pub signup_time: NaiveDateTime,
    pub role: i16
}

impl RUser {
    pub fn delete(conn: &PgConnection, id: Uuid) -> Result<usize, String> {
        let res = diesel::update(all_rusers.find(id))
            .set(ruser::status.eq(2))
            .execute(conn);
        match res {
            Ok(data) => Ok(data),
            Err(err) => Err(format!("{}", err))
        }
    }

    pub fn change_permission(conn: &PgConnection, data: ChangePermission) -> Result<usize, String> {
        let res = diesel::update(all_rusers.filter(ruser::id.eq(data.id)))
            .set((ruser::role.eq(data.permission)))
            .execute(conn);
        match res {
            Ok(num_update) => Ok(num_update),
            Err(err) => Err(format!("{}", err))
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChangePermission {
    pub id: Uuid,
    pub permission: i16,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoginUser {
    account: String,
    password: String,
    remember: bool
}

impl LoginUser {
    pub fn verification(&self, conn: &PgConnection, redis_pool: &Arc<RedisPool>, max_age: &Option<i64>) -> Result<String, String> {
        let res = all_rusers
            .filter(ruser::status.eq(0))
            .filter(ruser::account.eq(self.account.to_owned()))
            .get_result::<RawUser>(conn);
        match res {
            Ok(data) => {
                if data.password == sha3_256_encode(self.password.to_owned() + &data.salt) {
                    let ttl = match max_age {
                        &Some(t) => t * 3600,
                        &None => 24 * 60 * 60
                    };

                    let cookie = sha3_256_encode(random_string(8));
                    redis_pool.hset(&cookie, "login_time", Local::now().timestamp());
                    redis_pool.hset(&cookie, "info", json!(data.into_user_info()).to_string());
                    redis_pool.expire(&cookie, ttl);
                    Ok(cookie)
                } else {
                    Err(format!("用户或密码错误"))
                }
            }
            Err(err) => {
                Err(format!("{}", err))
            }
        }
    }

    pub fn get_remember(&self) -> bool {
        self.remember
    }

    pub fn sign_out(redis_pool: &Arc<RedisPool>, cookies: &str) -> bool {
        redis_pool.del(cookies)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EditUser {
    pub nickname: String,
    pub say: Option<String>,
    pub avatar: Option<String>,
    pub wx_openid: Option<String>,
}

impl EditUser {
    pub fn edit_user(self, conn: &PgConnection, redis_pool: &Arc<RedisPool>, cookie: &str) -> Result<usize, String> {
        let info = serde_json::from_str::<RUser>(&redis_pool.hget::<String>(cookie, "info")).unwrap();
        let res = diesel::update(all_rusers.filter(ruser::id.eq(info.id)))
            .set((ruser::nickname.eq(self.nickname), ruser::say.eq(self.say), ruser::avatar.eq(self.avatar), ruser::wx_openid.eq(self.wx_openid)))
            .get_result::<RawUser>(conn);
        match res {
            Ok(data) => {
                redis_pool.hset(cookie, "info", json!(data.into_user_info()).to_string());
                Ok(1)
            }
            Err(err) => Err(format!("{}", err))
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChangePassword {
    pub old_password: String,
    pub new_password: String
}

impl ChangePassword {
    pub fn change_password(&self, conn: &PgConnection, redis_pool: &Arc<RedisPool>, cookie: &str) -> Result<usize, String> {
        let info = serde_json::from_str::<RUser>(&redis_pool.hget::<String>(cookie, "info")).unwrap();

        if !self.verification(conn, &info.id) {
            return Err("Verification error".to_string())
        }

        let salt = random_string(6);
        let password = sha3_256_encode(self.new_password.to_owned() + &salt);
        let res = diesel::update(all_rusers.filter(ruser::id.eq(info.id)))
            .set((ruser::password.eq(&password), ruser::salt.eq(&salt)))
            .execute(conn);
        match res {
            Ok(num_update) => Ok(num_update),
            Err(err) => Err(format!("{}", err))
        }
    }

    fn verification(&self, conn: &PgConnection, id: &Uuid) -> bool {
        let old_user = all_rusers.filter(ruser::id.eq(id)).get_result::<RawUser>(conn);
        match old_user {
            Ok(old) => {
                if old.password == sha3_256_encode(self.old_password.to_owned() + &old.salt) {
                    true
                } else { false }
            }
            Err(_) => false
        }
    }
}

#[derive(Insertable, Debug, Clone, Deserialize, Serialize)]
#[table_name = "ruser"]
struct NewUser {
    pub account: String,
    pub password: String,
    pub salt: String,
    pub nickname: String,
}

impl NewUser {
    fn new(reg: RegisteredUser, salt: String) -> Self {
        NewUser {
            account: reg.account,
            password: reg.password,
            salt,
            nickname: reg.nickname,
        }
    }

    fn insert(&self, conn: &PgConnection, redis_pool: &Arc<RedisPool>) -> Result<String, String> {
        match diesel::insert_into(ruser::table)
            .values(self)
            .get_result::<RawUser>(conn) {
            Ok(info) => {
                let section = InsertSection {
                    title: info.nickname.clone(),
                    description: format!("{}的博客", info.nickname),
                    stype: 1,
                    suser: info.id,
                };
                section.insert(conn);
                self.set_cookies(redis_pool, info.into_user_info())
            }
            Err(err) => {
                Err(format!("{}", err))
            }
        }
    }

    fn set_cookies(&self, redis_pool: &Arc<RedisPool>, info: RUser) -> Result<String, String> {
        let cookie = sha3_256_encode(random_string(8));
        let redis_key = "user_".to_string() + &cookie;
        redis_pool.hset(&("user_".to_string() + &cookie), "login_time", Local::now().timestamp());
        redis_pool.hset(&redis_key, "info", json!(info).to_string());
        redis_pool.expire(&redis_key, 24 * 3600);
        Ok(cookie)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RegisteredUser {
    pub account: String,
    pub password: String,
    pub nickname: String,
}

impl RegisteredUser {
    pub fn register(self, conn: &PgConnection, redis_pool: &Arc<RedisPool>) -> Result<String, String> {
        NewUser::new(self, random_string(6)).insert(conn, redis_pool)
    }
}
