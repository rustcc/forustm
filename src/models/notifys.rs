use super::super::RedisPool;

use serde_json;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserNotify {
    pub user_id: Uuid,
    pub send_user_name: String,
    pub article_id: Uuid,
    pub article_title: String,
    pub notify_type: String,
}

impl UserNotify {
    pub fn cache(&self, redis_pool: &Arc<RedisPool>) {
        let content = serde_json::to_string(self).unwrap();
        let user_notify_key = format!("user:notify:{}", self.user_id.hyphenated().to_string());
        // remove old value
        redis_pool.lrem(&user_notify_key, 0, &content);
        // put new value to list top
        redis_pool.lpush(&user_notify_key, &content);
        // set expire time 15 day or increase expire time to 15 day
        const EXPIRE_TIME: i64 = 15 * 24 * 3600;
        redis_pool.expire(&user_notify_key, EXPIRE_TIME);
        // limit list size to 100
        redis_pool.ltrim(&user_notify_key, 0, 99);
    }

    pub fn get_notifys(user_id: Uuid, redis_pool: &Arc<RedisPool>) -> Option<Vec<UserNotify>> {
        let user_notify_key = format!("user:notify:{}", user_id.hyphenated().to_string());
        let notifys: Vec<String> = redis_pool.lrange(&user_notify_key, 0, -1);
        if notifys.len() > 0 {
            let notifys: Vec<UserNotify> = notifys
                .iter()
                .map(|notify_string| {
                    let user_notify: UserNotify = serde_json::from_str(&notify_string).unwrap();
                    user_notify
                })
                .collect();
            Some(notifys)
        } else {
            None
        }
    }

    pub fn remove_notifys_for_article(user_id: Uuid, article_id: Uuid, redis_pool: &Arc<RedisPool>) {
        let user_notify_key = format!("user:notify:{}", user_id.hyphenated().to_string());
        let notifys: Vec<String> = redis_pool.lrange(&user_notify_key, 0, -1);
        if notifys.len() > 0 {
            for notify_string in notifys {
                if notify_string.contains(&article_id.hyphenated().to_string()) {
                    redis_pool.lrem(&user_notify_key, 0, notify_string);
                }
            }
        }
    }
}
