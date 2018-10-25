rm .env && echo DATABASE_URL=postgres://postgres@"$(docker inspect -f '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' postgres)":5432/postgres > .env
docker build -t musl-rest:latest .
