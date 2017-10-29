-- Your SQL goes here
CREATE TABLE comment (
  id SERIAL PRIMARY KEY,
  content VARCHAR NOT NULL,
  article_id INTEGER NOT NULL,
  author_id INTEGER NOT NULL,
  created_time BIGINT NOT NULL
)
