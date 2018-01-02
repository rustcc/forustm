use super::super::ruser;
use super::super::ruser::dsl::ruser as all_rusers;
use super::super::{sha3_256_encode, random_string, RedisPool, InsertSection,
                   send_reset_password_email, get_github_primary_email};

use uuid::Uuid;
use chrono::{NaiveDateTime, Local};
use diesel;
use diesel::prelude::*;
use diesel::PgConnection;
use serde_json;
use std::sync::Arc;
use std::thread;
use super::ChangStatus;
use sapper::Client;


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
    pub status: i16, // 0 normal, 1 frozen, 2 deleted
    pub github: Option<String>,
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
            github: self.github,
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
    pub role: i16,
    pub github: Option<String>,
}

impl RUser {
    pub fn query_with_id(conn: &PgConnection, id: Uuid) -> Result<RUser, String> {
        let res = all_rusers.filter(ruser::id.eq(id)).first::<RawUser>(conn);
        match res {
            Ok(data) => Ok(data.into_user_info()),
            Err(e) => Err(format!("{}", e)),
        }
    }

    pub fn view_user_list(
        conn: &PgConnection,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Self>, String> {
        let res = all_rusers
            .limit(limit)
            .offset(offset)
            .order(ruser::signup_time)
            .get_results::<RawUser>(conn);
        match res {
            Ok(raw_user_list) => {
                Ok(
                    raw_user_list
                        .into_iter()
                        .map(|raw_user: RawUser| raw_user.into_user_info())
                        .collect::<Vec<Self>>(),
                )
            }
            Err(err) => Err(format!("{}", err)),
        }
    }

    pub fn change_status(conn: &PgConnection, data: ChangStatus) -> Result<usize, String> {
        let res = diesel::update(all_rusers.find(data.id))
            .set(ruser::status.eq(data.status))
            .execute(conn);
        match res {
            Ok(data) => Ok(data),
            Err(err) => Err(format!("{}", err)),
        }
    }

    pub fn change_permission(conn: &PgConnection, data: ChangePermission) -> Result<usize, String> {
        let res = diesel::update(all_rusers.filter(ruser::id.eq(data.id)))
            .set((ruser::role.eq(data.permission)))
            .execute(conn);
        match res {
            Ok(num_update) => Ok(num_update),
            Err(err) => Err(format!("{}", err)),
        }
    }

    pub fn view_with_cookie(redis_pool: &Arc<RedisPool>, cookie: &str) -> String {
        redis_pool.hget::<String>(cookie, "info")
    }

    pub fn reset_password(conn: &PgConnection, account: String) -> Result<usize, String> {
        let salt = random_string(6);
        let new_password = random_string(8);
        let res = diesel::update(all_rusers.filter(ruser::account.eq(&account)))
            .set((
                ruser::password.eq(
                    sha3_256_encode(new_password.clone() + &salt),
                ),
                ruser::salt.eq(salt),
            ))
            .execute(conn);
        match res {
            Ok(num) => {
                thread::spawn(move || send_reset_password_email(new_password, account));
                Ok(num)
            }
            Err(err) => Err(format!("{}", err)),
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
    remember: bool,
}

impl LoginUser {
    pub fn verification(
        &self,
        conn: &PgConnection,
        redis_pool: &Arc<RedisPool>,
        max_age: &Option<i64>,
    ) -> Result<String, String> {
        let res = all_rusers
            .filter(ruser::status.eq(0))
            .filter(ruser::account.eq(self.account.to_owned()))
            .get_result::<RawUser>(conn);
        match res {
            Ok(data) => {
                if data.password == sha3_256_encode(self.password.to_owned() + &data.salt) {
                    let ttl = match max_age {
                        &Some(t) => t * 3600,
                        &None => 24 * 60 * 60,
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
            Err(err) => Err(format!("{}", err)),
        }
    }

    pub fn get_remember(&self) -> bool {
        self.remember
    }

    pub fn sign_out(redis_pool: &Arc<RedisPool>, cookies: &str) -> bool {
        redis_pool.del(cookies)
    }

    pub fn login_with_github(
        conn: &PgConnection,
        redis_pool: &Arc<RedisPool>,
        https_client: &Client,
        github: String,
        nickname: String,
        token: String,
    ) -> Result<String, String> {
        let ttl = 24 * 60 * 60;
        match all_rusers
            .filter(ruser::status.eq(0))
            .filter(ruser::github.eq(&github))
            .get_result::<RawUser>(conn) {
            // github already exists
            Ok(data) => {
                let cookie = sha3_256_encode(random_string(8));
                redis_pool.hset(&cookie, "login_time", Local::now().timestamp());
                redis_pool.hset(&cookie, "info", json!(data.into_user_info()).to_string());
                redis_pool.expire(&cookie, ttl);
                Ok(cookie)
            }
            Err(_) => {
                let email = match get_github_primary_email(https_client, &token) {
                    Ok(data) => data,
                    Err(e) => return Err(e)
                };

                match all_rusers
                    .filter(ruser::status.eq(0))
                    .filter(ruser::account.eq(&email))
                    .get_result::<RawUser>(conn) {
                    // Account already exists but not linked
                    Ok(data) => {
                        let res = diesel::update(all_rusers.filter(ruser::id.eq(data.id)))
                            .set(ruser::github.eq(github))
                            .get_result::<RawUser>(conn);
                        match res {
                            Ok(info) => {
                                let cookie = sha3_256_encode(random_string(8));
                                redis_pool.hset(&cookie, "login_time", Local::now().timestamp());
                                redis_pool.hset(
                                    &cookie,
                                    "info",
                                    json!(info.into_user_info()).to_string(),
                                );
                                redis_pool.expire(&cookie, ttl);
                                Ok(cookie)
                            }
                            Err(err) => Err(format!("{}", err)),
                        }
                    }
                    // sign up
                    Err(_) => {
                        NewUser::new_with_github(email, github, nickname).insert(conn, redis_pool)
                    }
                }
            }
        }
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
    pub fn edit_user(
        self,
        conn: &PgConnection,
        redis_pool: &Arc<RedisPool>,
        cookie: &str,
    ) -> Result<usize, String> {
        let info = serde_json::from_str::<RUser>(&redis_pool.hget::<String>(cookie, "info"))
            .unwrap();
        let res = diesel::update(all_rusers.filter(ruser::id.eq(info.id)))
            .set((
                ruser::nickname.eq(self.nickname),
                ruser::say.eq(self.say),
                ruser::avatar.eq(self.avatar),
                ruser::wx_openid.eq(self.wx_openid),
            ))
            .get_result::<RawUser>(conn);
        match res {
            Ok(data) => {
                redis_pool.hset(cookie, "info", json!(data.into_user_info()).to_string());
                Ok(1)
            }
            Err(err) => Err(format!("{}", err)),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChangePassword {
    pub old_password: String,
    pub new_password: String,
}

impl ChangePassword {
    pub fn change_password(
        &self,
        conn: &PgConnection,
        redis_pool: &Arc<RedisPool>,
        cookie: &str,
    ) -> Result<usize, String> {
        let info = serde_json::from_str::<RUser>(&redis_pool.hget::<String>(cookie, "info"))
            .unwrap();

        if !self.verification(conn, &info.id) {
            return Err("Verification error".to_string());
        }

        let salt = random_string(6);
        let password = sha3_256_encode(self.new_password.to_owned() + &salt);
        let res = diesel::update(all_rusers.filter(ruser::id.eq(info.id)))
            .set((ruser::password.eq(&password), ruser::salt.eq(&salt)))
            .execute(conn);
        match res {
            Ok(num_update) => Ok(num_update),
            Err(err) => Err(format!("{}", err)),
        }
    }

    fn verification(&self, conn: &PgConnection, id: &Uuid) -> bool {
        let old_user = all_rusers.filter(ruser::id.eq(id)).get_result::<RawUser>(
            conn,
        );
        match old_user {
            Ok(old) => {
                if old.password == sha3_256_encode(self.old_password.to_owned() + &old.salt) {
                    true
                } else {
                    false
                }
            }
            Err(_) => false,
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
    pub github: Option<String>,
}

impl NewUser {
    fn new(reg: RegisteredUser, salt: String) -> Self {
        NewUser {
            account: reg.account,
            password: sha3_256_encode(reg.password + &salt),
            salt: salt,
            nickname: reg.nickname,
            github: None,
        }
    }

    fn new_with_github(email: String, github: String, nickname: String) -> Self {
        NewUser {
            account: email,
            password: sha3_256_encode(random_string(8)),
            salt: random_string(6),
            nickname: nickname,
            github: Some(github),
        }
    }

    fn insert(&self, conn: &PgConnection, redis_pool: &Arc<RedisPool>) -> Result<String, String> {
        match all_rusers
            .filter(ruser::account.eq(&self.account))
            .first::<RawUser>(conn) {
            Ok(_) => Err("Account already exists".to_string()),
            Err(_) => {
                match diesel::insert_into(ruser::table)
                    .values(self)
                    .get_result::<RawUser>(conn) {
                    Ok(info) => {
                        let section = InsertSection {
                            title: info.nickname.clone(),
                            description: format!("{}的博客", info.nickname),
                            stype: 1,
                            suser: Some(info.id),
                        };
                        section.insert(conn);
                        self.set_cookies(redis_pool, info.into_user_info())
                    }
                    Err(err) => Err(format!("{}", err)),
                }
            }
        }
    }

    fn set_cookies(&self, redis_pool: &Arc<RedisPool>, info: RUser) -> Result<String, String> {
        let cookie = sha3_256_encode(random_string(8));
        redis_pool.hset(&cookie, "login_time", Local::now().timestamp());
        redis_pool.hset(&cookie, "info", json!(info).to_string());
        redis_pool.expire(&cookie, 24 * 3600);
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
    pub fn register(
        self,
        conn: &PgConnection,
        redis_pool: &Arc<RedisPool>,
    ) -> Result<String, String> {
        NewUser::new(self, random_string(6)).insert(conn, redis_pool)
    }
}
