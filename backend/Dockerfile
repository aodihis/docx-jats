# ── Stage 1: dependency cache (cargo-chef) ───────────────────────────────────
FROM rust:1.88-slim-bookworm AS chef
RUN cargo install cargo-chef --locked
WORKDIR /app

FROM chef AS planner
COPY Cargo.toml Cargo.lock ./
COPY backend ./backend
RUN cargo chef prepare --recipe-path recipe.json

# ── Stage 2: build ────────────────────────────────────────────────────────────
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Compile dependencies only (cached as long as Cargo.lock doesn't change)
RUN cargo chef cook --release --recipe-path recipe.json
# Build the actual binary
COPY Cargo.toml Cargo.lock ./
COPY backend ./backend
RUN cargo build --release --bin docx-jats-backend

# ── Stage 3: minimal runtime image ───────────────────────────────────────────
FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get upgrade -y \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --system app && useradd --system --gid app app
WORKDIR /app
COPY --from=builder /app/target/release/docx-jats-backend ./docx-jats-backend
RUN chown app:app ./docx-jats-backend
ENV PORT=5505
ENV RUST_LOG=docx_jats_backend=info,tower_http=info
ENV LOG_FORMAT=json
EXPOSE 5505
USER app
CMD ["./docx-jats-backend"]
