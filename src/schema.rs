table! {
    article (id) {
        id -> Uuid,
        title -> Varchar,
        raw_content -> Varchar,
        content -> Varchar,
        section_id -> Uuid,
        author_id -> Uuid,
        tags -> Varchar,
        stype -> Int4,
        created_time -> Timestamp,
        status -> Int2,
    }
}

table! {
    comment (id) {
        id -> Uuid,
        content -> Varchar,
        article_id -> Uuid,
        author_id -> Uuid,
        created_time -> Timestamp,
        status -> Int2,
    }
}

table! {
    ruser (id) {
        id -> Uuid,
        account -> Varchar,
        password -> Varchar,
        salt -> Varchar,
        nickname -> Varchar,
        avatar -> Nullable<Varchar>,
        wx_openid -> Nullable<Varchar>,
        say -> Nullable<Varchar>,
        signup_time -> Timestamp,
        role -> Int2,
        status -> Int2,
        github -> Nullable<Varchar>,
    }
}

table! {
    section (id) {
        id -> Uuid,
        title -> Varchar,
        description -> Varchar,
        stype -> Int4,
        suser -> Nullable<Uuid>,
        created_time -> Timestamp,
        status -> Int2,
    }
}

joinable!(article -> ruser (author_id));
joinable!(article -> section (section_id));
joinable!(comment -> article (article_id));
joinable!(comment -> ruser (author_id));
joinable!(section -> ruser (suser));

allow_tables_to_appear_in_same_query!(
    article,
    comment,
    ruser,
    section,
);
