use super::super::util::{
    random_string, 
    sha3_256_encode};

use super::super::thirdparts::{
    inner_get_github_primary_email, 
    send_reset_password_email};

use super::super::db::RedisPool;

use schema::ruser as ruser_schema;
use schema::ruser::table as ruser_table;



use std::sync::Arc;
use std::thread;
use uuid::Uuid;
use chrono::{Local, NaiveDateTime};
use serde_json;

use diesel;
use diesel::PgConnection;
use diesel::prelude::*;

//
// MODEL
//

#[derive(Queryable)]
struct RUser {
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
    pub role: i16,   // member => 2, manager => 1, admin => 0
    pub status: i16, // 0 normal, 1 frozen, 2 deleted
    pub github: Option<String>,
}

impl RUser {
    fn into_user_info(self) -> RUserDto {
        RUserDto {
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


#[derive(Insertable, Debug, Clone, Deserialize, Serialize)]
#[table_name = "ruser"]
struct NewUserDmo {
    pub account: String,
    pub password: String,
    pub salt: String,
    pub nickname: String,
    pub github: Option<String>,
}

impl NewUserDmo {
    fn new(reg: RegisteredUser, salt: String) -> Self {
        NewUser {
            account: reg.account,
            password: sha3_256_encode(&format!("{}{}", reg.password, salt)),
            salt: salt,
            nickname: reg.nickname,
            github: None,
        }
    }

    fn new_with_github(email: String, github: String, nickname: String) -> Self {
        NewUser {
            account: email,
            password: sha3_256_encode(&random_string(8)),
            salt: random_string(6),
            nickname: nickname,
            github: Some(github),
        }
    }

    fn insert(&self, conn: &PgConnection, redis_pool: &Arc<RedisPool>) -> Result<String, String> {
        match ruser_table
            .filter(ruser_schema::account.eq(&self.account))
            .first::<RawUser>(conn)
        {
            Ok(_) => Err("Account already exists".to_string()),
            Err(_) => match diesel::insert_into(ruser_table)
                .values(self)
                .get_result::<RawUser>(conn)
            {
                Ok(info) => {
                    let section = InsertSection {
                        title: info.nickname.clone(),
                        description: format!("{}'s blog", info.nickname),
                        stype: 1,
                        suser: Some(info.id),
                    };
                    section.insert(conn);
                    self.set_cookies(redis_pool, &info.into_user_info())
                }
                Err(err) => Err(format!("{}", err)),
            },
        }
    }

    fn set_cookies(&self, redis_pool: &Arc<RedisPool>, info: &RUser) -> Result<String, String> {
        let cookie = sha3_256_encode(&random_string(8));
        redis_pool.hset(&cookie, "login_time", Local::now().timestamp());
        redis_pool.hset(&cookie, "info", json!(info).to_string());
        redis_pool.expire(&cookie, 24 * 3600);
        Ok(cookie)
    }
}

//
// DTOs
//

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RUserDto {
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

impl RUserDto {
    pub fn query_with_id(conn: &PgConnection, id: Uuid) -> Result<RUserDto, String> {
        let res = ruser_table.filter(ruser_schema::id.eq(id)).first::<RUser>(conn);
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
        let res = ruser_table
            .limit(limit)
            .offset(offset)
            .order(ruser_schema::signup_time)
            .get_results::<RawUser>(conn);
        match res {
            Ok(raw_user_list) => Ok(raw_user_list
                .into_iter()
                .map(|raw_user: RawUser| raw_user.into_user_info())
                .collect::<Vec<Self>>()),
            Err(err) => Err(format!("{}", err)),
        }
    }

    pub fn change_status(conn: &PgConnection, data: &ChangStatusDmo) -> Result<usize, String> {
        let res = diesel::update(ruser_table.find(data.id))
            .set(ruser_schema::status.eq(data.status))
            .execute(conn);
        match res {
            Ok(data) => Ok(data),
            Err(err) => Err(format!("{}", err)),
        }
    }

    pub fn change_permission(
        conn: &PgConnection,
        data: &ChangePermission,
    ) -> Result<usize, String> {
        let res = diesel::update(ruser_table.filter(ruser_schema::id.eq(data.id)))
            .set(ruser_schema::role.eq(data.permission))
            .execute(conn);
        match res {
            Ok(num_update) => Ok(num_update),
            Err(err) => Err(format!("{}", err)),
        }
    }

    pub fn view_with_cookie(redis_pool: &Arc<RedisPool>, cookie: &str) -> String {
        redis_pool.hget::<String>(cookie, "info")
    }

    pub fn send_reset_pwd_email(
        conn: &PgConnection,
        redis_pool: &Arc<RedisPool>,
        account: String,
    ) -> Result<(), String> {
        let res = ruser_table
            .filter(ruser_schema::status.eq(0))
            .filter(ruser_schema::account.eq(&account))
            .get_result::<RawUser>(conn);
        match res {
            Ok(data) => {
                let cookie = sha3_256_encode(&random_string(8));
                redis_pool.hset(&cookie, "info", json!(data.into_user_info()).to_string());
                redis_pool.expire(&cookie, 60 * 10);
                thread::spawn(move || send_reset_password_email(&cookie, &account));
                Ok(())
            }
            Err(err) => Err(format!("{}", err)),
        }
    }

    pub fn reset_pwd(
        conn: &PgConnection,
        redis_pool: &Arc<RedisPool>,
        pwd: String,
        cookie: String,
    ) -> Result<String, String> {
        let info =
            serde_json::from_str::<RUser>(&redis_pool.hget::<String>(&cookie, "info")).unwrap();
        let salt = random_string(6);
        let password = sha3_256_encode(&format!("{}{}", pwd, salt));
        let res = diesel::update(ruser_table.filter(ruser_schema::id.eq(info.id)))
            .set((ruser_schema::password.eq(&password), ruser_schema::salt.eq(&salt)))
            .execute(conn);
        match res {
            Ok(_) => {
                redis_pool.expire(&cookie, 24 * 60 * 60);
                Ok(cookie)
            }
            Err(err) => Err(format!("{}", err)),
        }
    }
}


#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChangePermissionDmo {
    pub id: Uuid,
    pub permission: i16,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoginUserDto {
    account: String,
    password: String,
    remember: bool,
}

impl LoginUserDto {
    pub fn verification(
        &self,
        conn: &PgConnection,
        redis_pool: &Arc<RedisPool>,
        max_age: &Option<i64>,
    ) -> Result<String, String> {
        let res = ruser_table
            .filter(ruser_schema::status.eq(0))
            .filter(ruser_schema::account.eq(self.account.to_owned()))
            .get_result::<RawUser>(conn);
        match res {
            Ok(data) => {
                if data.password == sha3_256_encode(&format!("{}{}", self.password, data.salt)) {
                    let ttl = match *max_age {
                        Some(t) => t * 3600,
                        None => 24 * 60 * 60,
                    };

                    let cookie = sha3_256_encode(&random_string(8));
                    redis_pool.hset(&cookie, "login_time", Local::now().timestamp());
                    redis_pool.hset(&cookie, "info", json!(data.into_user_info()).to_string());
                    redis_pool.expire(&cookie, ttl);
                    Ok(cookie)
                } else {
                    Err("account or password error".into())
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
        github: String,
        nickname: String,
        token: &str,
    ) -> Result<String, String> {
        let ttl = 24 * 60 * 60;
        match ruser_table
            .filter(ruser_schema::status.eq(0))
            .filter(ruser_schema::github.eq(&github))
            .get_result::<RawUser>(conn)
        {
            // github already exists
            Ok(data) => {
                let cookie = sha3_256_encode(&random_string(8));
                redis_pool.hset(&cookie, "login_time", Local::now().timestamp());
                redis_pool.hset(&cookie, "info", json!(data.into_user_info()).to_string());
                redis_pool.expire(&cookie, ttl);
                Ok(cookie)
            }
            Err(_) => {
                let email = match inner_get_github_primary_email(token) {
                    Ok(data) => data,
                    Err(e) => return Err(e),
                };

                match ruser_table
                    .filter(ruser_schema::status.eq(0))
                    .filter(ruser_schema::account.eq(&email))
                    .get_result::<RawUser>(conn)
                {
                    // Account already exists but not linked
                    Ok(data) => {
                        let res = diesel::update(ruser_table.filter(ruser_schema::id.eq(data.id)))
                            .set(ruser_schema::github.eq(github))
                            .get_result::<RawUser>(conn);
                        match res {
                            Ok(info) => {
                                let cookie = sha3_256_encode(&random_string(8));
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
pub struct EditUserDmo {
    pub nickname: String,
    pub say: Option<String>,
    pub avatar: Option<String>,
    pub wx_openid: Option<String>,
}

impl EditUserDmo {
    pub fn edit_user(
        self,
        conn: &PgConnection,
        redis_pool: &Arc<RedisPool>,
        cookie: &str,
    ) -> Result<usize, String> {
        let info =
            serde_json::from_str::<RUser>(&redis_pool.hget::<String>(cookie, "info")).unwrap();
        let res = diesel::update(ruser_table.filter(ruser_schema::id.eq(info.id)))
            .set((
                ruser_schema::nickname.eq(self.nickname),
                ruser_schema::say.eq(self.say),
                ruser_schema::avatar.eq(self.avatar),
                ruser_schema::wx_openid.eq(self.wx_openid),
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
pub struct ChangePasswordDmo {
    pub old_password: String,
    pub new_password: String,
}

impl ChangePasswordDmo {
    pub fn change_password(
        &self,
        conn: &PgConnection,
        redis_pool: &Arc<RedisPool>,
        cookie: &str,
    ) -> Result<usize, String> {
        let info =
            serde_json::from_str::<RUser>(&redis_pool.hget::<String>(cookie, "info")).unwrap();

        if !self.verification(conn, &info.id) {
            return Err("Verification error".to_string());
        }

        let salt = random_string(6);
        let password = sha3_256_encode(&format!("{}{}", self.new_password, salt));
        let res = diesel::update(ruser_table.filter(ruser_schema::id.eq(info.id)))
            .set((ruser_schema::password.eq(&password), ruser_schema::salt.eq(&salt)))
            .execute(conn);
        match res {
            Ok(num_update) => Ok(num_update),
            Err(err) => Err(format!("{}", err)),
        }
    }

    fn verification(&self, conn: &PgConnection, id: &Uuid) -> bool {
        let old_user = ruser_table
            .filter(ruser_schema::id.eq(id))
            .get_result::<RawUser>(conn);
        match old_user {
            Ok(old) => {
                old.password == sha3_256_encode(&format!("{}{}", self.old_password, old.salt))
            }
            Err(_) => false,
        }
    }
}


#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RegisteredUserDmo {
    pub account: String,
    pub password: String,
    pub nickname: String,
}

impl RegisteredUserDmo {
    pub fn register(
        self,
        conn: &PgConnection,
        redis_pool: &Arc<RedisPool>,
    ) -> Result<String, String> {
        NewUser::new(self, random_string(6)).insert(conn, redis_pool)
    }
}


#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChangStatusDmo {
    pub id: Uuid,
    pub status: i16,
}




/// get user if login or none
pub fn get_ruser_from_session(req: &Request) -> Option<RUserDto> {
    let redis_pool = req.ext().get::<Redis>().unwrap();
    match req.ext().get::<SessionVal>() {
        Some(cookie) => {
            if redis_pool.exists(cookie) {
                let redis_pool = req.ext().get::<Redis>().unwrap();
                let user: RUserDto =
                    serde_json::from_str(&RUserDto::view_with_cookie(redis_pool, cookie)).unwrap();
                Some(user)
            } else {
                None
            }
        }
        None => None,
    }
}

/// get request's real ip when request proxyed by nginx or normal ip
pub fn get_real_ip_from_req(req: &Request) -> Option<String> {
    match req.headers().get_raw("X-Real-IP") {
        Some(fip) => String::from_utf8((*fip)[0].clone()).ok(),
        None => serde_json::to_string(&req.remote_addr().ip())
            .ok()
            .map(|s| String::from(&s[1..s.len() - 1])),
    }
}

/// get request's user-agent
pub fn get_user_agent_from_req(req: &Request) -> Option<String> {
    req.headers()
        .get::<UserAgent>()
        .map(|user_agent| String::from(user_agent.trim()))
}
