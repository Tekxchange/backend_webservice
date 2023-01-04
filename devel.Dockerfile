FROM rust:latest

WORKDIR /usr/src/app

RUN cargo install cargo-watch

CMD ["cargo", "watch", "-x", "run"]