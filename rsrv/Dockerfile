FROM alpine:latest
WORKDIR /usr/local/bin
COPY serve .
COPY .env .
ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=8000
RUN apk add --no-cache postgresql-libs libgcc
EXPOSE 8000
