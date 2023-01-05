FROM rust:latest

USER 1000:1000

RUN cargo install cargo-watch

VOLUME [ "/usr/src/app" ]

WORKDIR /usr/src/app

CMD ["cargo", "watch", "-x", "run"]