-- Your SQL goes here

CREATE TABLE comment (
  id uuid primary key default gen_random_uuid(),
  content VARCHAR NOT NULL,
  article_id uuid references article (id) not null,
  author_id uuid references ruser (id) not null,
  created_time timestamp not null default current_timestamp
)