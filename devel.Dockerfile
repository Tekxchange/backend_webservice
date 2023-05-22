
# --Base--
FROM rust:alpine as dependencies

WORKDIR /usr/src/app

RUN apk add libc6-compat libssl1.1 openssl-dev libgcc gcompat musl-dev

# --Dev Dependencies--
FROM dependencies as dev-dependencies

RUN cargo install cargo-watch

# --Localdev--
FROM dev-dependencies as localdev

EXPOSE 8000

CMD ["cargo-watch", "-x run"]

# --Release Builder--
FROM dependencies as release-builder

WORKDIR /usr/src/app
COPY . .

RUN cargo build --release

# --Release--
FROM dependencies as release

COPY --from=release-builder /usr/src/app/target/release/rust_tekxchange_backend .
COPY --from=release-builder /usr/src/app/Rocket.toml .
RUN chmod +x rust_tekxchange_backend

EXPOSE 8000

CMD ["./rust_tekxchange_backend"]