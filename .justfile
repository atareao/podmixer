user    := "atareao"
name    := `basename ${PWD}`
version := `git tag -l  | tail -n1`

default:
    @just --list

build:
    echo {{version}}
    echo {{name}}
    docker build -t {{user}}/{{name}}:{{version}} \
                 -t {{user}}/{{name}}:latest \
                 .

tag:
    docker tag {{user}}/{{name}}:{{version}} {{user}}/{{name}}:latest

push:
    docker push {{user}}/{{name}} --all-tags

run:
    docker run --rm \
               --init \
               -e RUST_LOG='debug' \
               -e RUST_ENV='production' \
               -e DB_URL='/app/db/podmixer.db' \
               -v db:/app/db \
               --publish '6996:6996' \
               --name {{name}} \
               {{user}}/{{name}}

sql sql:
    @echo "sql: {{sql}}"
    docker run --rm \
               --init \
               -it \
               -e DB_URL='/app/db/podmixer.db' \
               -v db:/app/db \
               --name podmixerdb \
               {{user}}/{{name}} \
               sqlite3 /app/db/podmixer.db "{{sql}}"
exe:
    docker run --rm \
               --init \
               -it \
               -e RUST_LOG='debug' \
               -e RUST_ENV='production' \
               -e DB_URL='/app/db/podmixer.db' \
               -v db:/app/db \
               --publish '6996:6996' \
               --name {{name}} \
               {{user}}/{{name}} \
               sh

test:
    echo {{version}}
    echo {{name}}
    docker build -t {{user}}/{{name}}:test \
                 .
    docker push {{user}}/{{name}}:test

