-- Your SQL goes here
CREATE TABLE ruser (
  id SERIAL PRIMARY KEY,
  account VARCHAR NOT NULL,
  password VARCHAR NOT NULL,
  salt VARCHAR NOT NULL,
  nickname VARCHAR NOT NULL,
  avatar VARCHAR NOT NULL,
  wx_openid VARCHAR NOT NULL,
  say VARCHAR NOT NULL,
  signup_time BIGINT NOT NULL
)
