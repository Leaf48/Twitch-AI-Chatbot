FROM rust:1.82-bookworm AS builder

WORKDIR /app
COPY src /app/src
COPY Cargo.lock Cargo.toml /app/

RUN cargo fetch --locked
RUN cargo build --release --locked

FROM debian:bookworm-slim AS runner

WORKDIR /app

RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/Twitch-AI-Chatbot /app/
COPY instructions /app/instructions
COPY config*.yml /app/

CMD [ "./Twitch-AI-Chatbot" ]