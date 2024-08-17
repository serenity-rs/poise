FROM rust:1.80 as build

WORKDIR /codebase


# Copy the source files
COPY . /codebase

# Build the project
RUN cargo build --package poise --example feature_showcase --release
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y openssl ca-certificates
COPY --from=build codebase/target/release//examples/feature_showcase /usr/local/bin/feature_showcase
CMD ["feature_showcase"]