FROM rust:1.82.0 AS builder

WORKDIR /workdir

COPY . .

RUN cargo build --release

FROM rust:1.82.0


COPY --from=builder /workdir/target/release/engine-tt .

EXPOSE 3000

ENTRYPOINT [ "./engine-tt" ]