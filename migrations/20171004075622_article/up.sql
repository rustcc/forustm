-- Your SQL goes here
CREATE TABLE article (
  id uuid primary key default gen_random_uuid(),
  title VARCHAR NOT NULL,
  raw_content VARCHAR NOT NULL,
  content VARCHAR NOT NULL,
  section_id uuid references section (id) not null,
  author_id uuid references ruser (id) not null,
  tags VARCHAR NOT NULL,
  created_time timestamp not null default current_timestamp
)
