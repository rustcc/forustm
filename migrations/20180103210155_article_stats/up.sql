-- Your SQL goes here
CREATE TABLE article_stats (
  id uuid primary key default gen_random_uuid(),
  article_id uuid references article (id) not null,
  created_time timestamp not null default current_timestamp,
  ruser_id uuid references ruser (id),
  user_agent varchar,
  visitor_ip varchar
);