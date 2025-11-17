FROM lukemathwalker/cargo-chef:latest AS chef
WORKDIR /app
RUN apt update && apt install lld clang -y

# Planner
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Builder
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
ENV SQLX_OFFLINE=true
RUN cargo build --release --bin zero2prod

# Runtime
FROM debian:trixie-slim AS runtime
WORKDIR /app
RUN apt-get update -y \
  && apt-get install -y --no-install-recommends openssl ca-certificates clang lld \
  # Clean up
  && apt-get autoremove -y \
  && apt-get clean -y \
  && rm -rf /var/lib/apt/lists/*

COPY --from=builder  /app/target/release/zero2prod zero2prod
COPY configuration configuration
ENV APP_ENVIRONMENT=local
# EXPOSE 3000

ENTRYPOINT [ "./zero2prod" ]
