pub mod articles;
pub mod comments;
pub mod sections;
pub mod rusers;

pub(crate) use self::articles::{Article, ArticleBrief, NewArticle, EditArticle, DeleteArticle};
pub(crate) use self::comments::{NewComment, DeleteComment, CommentWithNickName};
pub(crate) use self::rusers::{RUser, LoginUser, EditUser, ChangePassword, ChangePermission,
                              RegisteredUser};
pub(crate) use self::sections::{InsertSection, PubNotice, Section};

use uuid::Uuid;


#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChangStatus {
    pub id: Uuid,
    pub status: i16,
}
