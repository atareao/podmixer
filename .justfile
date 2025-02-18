user    := "atareao"
name    := `basename ${PWD}`
version := `git tag -l  | tail -n1`


list:
    @just --list

dev:
    cd front && pnpm i && pnpm run build && rm -rf ../back/static && mkdir ../back/static && cp -r ./dist/* ../back/static
    cd back && RUST_LOG=debug cargo run

front:
    cd front && pnpm run dev

back:
    cd back && RUST_LOG=debug cargo run

build:
    @docker build --tag={{user}}/{{name}}:{{version}} .

push:
    @docker push {{user}}/{{name}}:{{version}}

