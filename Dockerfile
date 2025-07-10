FROM lukemathwalker/cargo-chef:0.1.72-rust-1.88.0-slim-bullseye AS chef
WORKDIR /app

#
# Cache dependencies
#

FROM chef AS planner
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
COPY ./backend/ ./backend/
COPY ./backend_core/ ./backend_core/
RUN cargo chef prepare --bin backend --recipe-path recipe.json

#
# Cache dependencies
#

FROM chef AS builder 

COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --bin backend --release --recipe-path recipe.json
COPY --from=planner /app/Cargo.lock ./Cargo.lock
COPY --from=planner /app/Cargo.toml ./Cargo.toml
COPY --from=planner /app/backend/ ./backend/
COPY --from=planner /app/backend_core/ ./backend_core/
RUN cargo build --bin backend --release

#
# The final image.
#

FROM debian:bullseye-slim AS runtime

WORKDIR /app
EXPOSE 9999
COPY --from=builder /app/target/release/backend /bin/backend

ENTRYPOINT ["/bin/backend"]