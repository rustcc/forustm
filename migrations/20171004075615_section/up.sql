-- Your SQL goes here

CREATE TABLE section (
  id uuid primary key default gen_random_uuid(),
  title VARCHAR NOT NULL,
  description VARCHAR NOT NULL,
  stype INTEGER NOT NULL,
  suser uuid references ruser (id),
  created_time timestamp not null default current_timestamp,
  status smallint not null default 0
);