# nginx 配置模板

```
user www-data;
worker_processes 4;
pid /run/nginx.pid;
events {
        worker_connections 65535;
        # multi_accept on;
}

http{
    include       mime.types;
     upstream forustm_web {
        server 0.0.0.0:8081;
     }
     server {
        listen       80;
        server_name  0.0.0.0;

        location / {
            proxy_pass http://forustm_web;
            proxy_redirect off;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        }

        location /api/v1/ {
            proxy_pass http://0.0.0.0:8888/;
            proxy_redirect off;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        }
    }
}
```
