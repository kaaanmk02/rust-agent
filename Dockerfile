FROM rust:1.83-bookworm AS build
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=build /app/target/release/rust-agent /app/rust-agent
COPY gateway.json /app/gateway.json
EXPOSE 9090
ENV RUST_LOG=info
ENTRYPOINT ["/app/rust-agent"]
CMD ["gateway"]
