FROM rust:alpine as BUILDER

WORKDIR /usr/src/app

COPY . .

RUN apk add libc6-compat libressl-dev libgcc gcompat musl-dev
RUN cargo build --release

FROM alpine:latest as RUNNER

WORKDIR /usr/src/app

COPY --from=BUILDER /usr/src/app/target/release/rust_tekxchange_backend ./
COPY --from=BUILDER /usr/src/app/Rocket.toml ./

RUN apk add libc6-compat libssl1.1 libgcc gcompat
RUN chmod +x ./rust_tekxchange_backend

EXPOSE 8000

CMD ["./rust_tekxchange_backend"]