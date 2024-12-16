FROM rust:1.82.0 AS builder

WORKDIR /workdir

COPY dummy.rs .
COPY Cargo.toml Cargo.lock ./

RUN sed -i 's#src/main.rs#dummy.rs#' Cargo.toml
RUN cargo build --release
RUN sed -i 's#dummy.rs#src/main.rs#' Cargo.toml

COPY . .

RUN cargo build --release

FROM rust:1.82.0


COPY --from=builder /workdir/target/release/engine-tt .

EXPOSE 3000

ENTRYPOINT [ "./engine-tt" ]