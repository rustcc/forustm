-- Your SQL goes here
CREATE TABLE section (
  id SERIAL PRIMARY KEY,
  title VARCHAR NOT NULL,
  stype INTEGER NOT NULL,
  created_time BIGINT NOT NULL
)
