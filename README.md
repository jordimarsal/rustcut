# rustcut
A tiny Rust URL shortener with Actix-Web and SQLite


## Local test
User GET:<br>
curl -X GET http://localhost:8083/users

User POST:<br>
curl -X POST -H "Content-Type: application/json" -d '{"username":"example", "email":"example@mail.com"}' http://localhost:8083/users

User DELETE:<br>
curl -X DELETE http://localhost:8083/users/3