# 1. Pin specific version and OS (Bookworm matches the runtime image)
FROM rust:1.91-bookworm AS builder


# 2. Setup the directory structure
# We work in /usr/src so 'ohlg' and 'tfhe-rs' are siblings
WORKDIR /usr/src

# 3. Copy dependencies first
COPY tfhe-rs ./tfhe-rs

# 4. Copy the ohlg project
COPY ohlg ./ohlg

# 5. Build the ohlg project
WORKDIR /usr/src/ohlg
RUN cargo build --release

# 6. Create the runtime image
FROM debian:bookworm-slim

WORKDIR /app

# 7. Copy the binaries
COPY --from=builder /usr/src/ohlg/target/release/ohlg .
COPY --from=builder /usr/src/ohlg/target/release/server_odm .
COPY --from=builder /usr/src/ohlg/target/release/client_odm .
COPY --from=builder /usr/src/ohlg/target/release/verif_odm .
COPY --from=builder /usr/src/ohlg/target/release/shortint .
COPY --from=builder /usr/src/ohlg/target/release/longint .

CMD ["sh", "-c", "echo 'Available binaries: ohlg, client_odm, server_odm, verif_odm, shortint, longint'; exec sh"]

