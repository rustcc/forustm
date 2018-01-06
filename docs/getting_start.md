# Getting Start

## 如何在本地启动本项目

### 配置环境

环境要求：

- `Postgresql 9.6+`
- `Redis 3.0+`
- `Stable Rust 1.22 +`

项目启动：

```rust
forustm $  cargo build
forustm $  diesel setup
forustm $  cargo run --bin forustm_web
forustm $  cargo run --bin forustm_api
```
配置文件：

- .env
- 配置Nginx（[nginx配置文件](./nginx_template.md)）或Caddy

### Docker里运行Postgresql指南

- 安装`docker`
- 执行命令 : `docker pull postgres`，默认安装latest版本，[Docker Hub地址](https://hub.docker.com/_/postgres/)
- 执行命令 : `docker run --name your_postgres_name -e POSTGRES_PASSWORD=system -d -p 5432:5432 postgres`，启动postresql实例
- 登录到Postgresql： `docker run -it --rm --link your_postgres_name:postgres postgres  psql -h postgres -U postgres `，使用`\q`命令退出

### Get started with Docker Compose

Prepare the dependents

```
$ cargo install diesel_cli
```

Setup the DB

```
$ cd docker && docker-compose up -d
$ diesel setup
```

Compile here

```
$ cargo build
```

Restart

```
$ cd docker && docker-compose restart
```

Now, [Click Here](http://localhost)
