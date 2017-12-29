-- Your SQL goes here

create extension pgcrypto;

CREATE TABLE ruser (
  id uuid primary key default gen_random_uuid(),
  account VARCHAR unique NOT NULL,
  password VARCHAR NOT NULL,
  salt VARCHAR NOT NULL,
  nickname VARCHAR NOT NULL,
  avatar VARCHAR,
  wx_openid VARCHAR,
  say VARCHAR,
  signup_time timestamp not null default current_timestamp,
  role smallint not null default 2,
  status smallint not null default 0
);

Create index user_account on ruser (account);

insert into ruser (account, password, salt, role, nickname) values
('admin@admin.com', '325c162157dea106ce5bacc705c4929e4ec526a0290bfaba2dcbbf18103c7c2b', 'MKsiaw', 0, 'admin');
