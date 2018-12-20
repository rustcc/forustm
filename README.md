# Forustm

Forustm for Rust Community, developed in Rust, with [Sapper](https://github.com/sappworks/sapper).

Now works on [Rust.cc](https://rust.cc)

## How to run it

### Prepare

1. postgresql 9.6+;
2. redis;
3. configure pg and redis connection addresses and ports in .env;
4. execute `cargo install diesel_cli`;

### Steps

1. execute `diesel setup`;
2. `cargo build`;
3. configure nginx as the `docs/template_nginx.conf` discribes, and start nginx;
4. `cargo run --bin forustm_web`;
5. open a new tab, and `cargo run --bin forustm_api`;
6. visit `http://localhost`;

### Configurate Index Page Sections

1. login with `admin@admin.com` (default password is `admin`, you can modify it after logined);
2. visit `http://localhost/admin/section` to create new sections;
3. use `psql` to find the ids(uuids) of new sections in pg;
4. use `redis-cli`, connected to redis, `select 1`, and `rpush cate_sections {uuid}`, repeat it;
4. and `rpush proj_sections {uuid}`, repeat it;

### Configurate Index Page Public Notice

1. in redis, `select 1`;
2. `hset pub_notice`title xxxxxxxxxx`;
3. `hset pub_notice desc xxxxxxxxxxxxxxxxxxxxx`;

Now, visit `http://localhost` again. That's all.

Good lucky!

