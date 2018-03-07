use schema;

pub mod article;
pub mod comment;
pub mod section;
pub mod ruser;
pub mod articles_stat;
pub mod notify;

pub(crate) use self::articles::{Article, ArticleBrief, DeleteArticle, EditArticle, NewArticle,
                                SimpleArticle};
pub(crate) use self::comments::{CommentWithNickName, DeleteComment, NewComment};
pub(crate) use self::rusers::{ChangePassword, ChangePermission, EditUser, LoginUser, RUser,
                              RegisteredUser};
pub(crate) use self::sections::{InsertSection, PubNotice, Section};
pub(crate) use self::articles_stats::NewArticleStats;
pub(crate) use self::notifys::UserNotify;

use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChangStatus {
    pub id: Uuid,
    pub status: i16,
}
