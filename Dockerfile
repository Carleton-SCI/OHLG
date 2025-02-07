FROM rust:latest AS builder
WORKDIR /usr/src/ohlg
COPY ohlg .

WORKDIR /usr/src/tfhe-rs
COPY tfhe-rs .

WORKDIR /usr/src/ohlg
RUN cargo build --release

FROM debian:bookworm-slim

WORKDIR /app


COPY --from=builder /usr/src/ohlg/target/release/ohlg .
COPY --from=builder /usr/src/ohlg/target/release/server_odm .
COPY --from=builder /usr/src/ohlg/target/release/client_odm .
COPY --from=builder /usr/src/ohlg/target/release/verif_odm .

CMD ["sh", "-c", "echo 'Available binaries: ohlg, client_odm, server_odm, verif_odm'; exec sh"]

