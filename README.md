[![codecov](https://codecov.io/gh/jordimarsal/rustcut/branch/main/graph/badge.svg)](https://codecov.io/gh/jordimarsal/rustcut)

# rustcut
A tiny Rust URL shortener with Actix-Web and SQLite


## Local test
#### User GET:<br>
```sh
curl -X GET http://localhost:8083/users
```

#### User POST:<br>
```sh
curl -X POST -H "Content-Type: application/json" -d '{"username":"example", "email":"example@mail.com"}' http://localhost:8083/users
```

#### User DELETE:<br>
```sh
curl -X DELETE http://localhost:8083/users/3
```

#### URL create POST<br>
```sh
curl -X POST -H "Content-Type: application/json" -d '{"api_key":"1234567890", "target_url":"https://github.com/jordimarsal/rustcut"}' http://localhost:8083/url
```

#### URL redirect GET<br>
_Must be used from web browser. Url key must be some obtained from URL create POST_
```sh
curl -X GET \
http://localhost:8083/4gefwf31
```