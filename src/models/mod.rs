pub mod articles;
pub mod comments;
pub mod sections;
pub mod rusers;
pub mod articles_stats;

pub(crate) use self::articles::{Article, ArticleBrief, DeleteArticle, EditArticle, NewArticle};
pub(crate) use self::comments::{CommentWithNickName, DeleteComment, NewComment};
pub(crate) use self::rusers::{ChangePassword, ChangePermission, EditUser, LoginUser, RUser, RegisteredUser};
pub(crate) use self::sections::{InsertSection, PubNotice, Section};
pub(crate) use self::articles_stats::{NewArticleStats};

use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChangStatus {
    pub id: Uuid,
    pub status: i16,
}
