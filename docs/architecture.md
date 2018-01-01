# 论坛架构

```
                    ----------------> forustm_web ---------------------------
                    |                     |                                 |
                    |                     |                                 |
                    |                     |                                \|/
HTTP Request ---> Nginx                  cookies --------> Redis        ORM(diesel) -> Postgresql
                    |                                       /|\            /|\
                    |                cookies/data  ----------|              |
                    |                     |                                 |
                    ---------------> forustm_api ----------------------------




```
