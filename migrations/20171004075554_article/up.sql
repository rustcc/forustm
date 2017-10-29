-- Your SQL goes here
CREATE TABLE article (
  id SERIAL PRIMARY KEY,
  title VARCHAR NOT NULL,
  content VARCHAR NOT NULL,
  section_id INTEGER NOT NULL,
  author_id INTEGER NOT NULL,
  tags VARCHAR NOT NULL,
  created_time BIGINT NOT NULL
)
