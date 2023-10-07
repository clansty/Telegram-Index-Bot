FROM rust:1.72-bookworm as build

# create a new empty shell project
RUN USER=root cargo new --bin telegram-index
WORKDIR /telegram-index

# copy over your manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# this build step will cache your dependencies
RUN cargo build --release
RUN rm src/*.rs

# copy your source tree
COPY ./src ./src

# build for release
RUN rm ./target/release/deps/telegram*
RUN cargo build --release

# our final base
FROM debian:bookworm-slim

RUN apt-get update && apt install -y openssl ca-certificates && apt-get clean autoclean && rm -rf /var/lib/{apt,dpkg,cache,log}/

# copy the build artifact from the build stage
COPY --from=build /telegram-index/target/release/telegram-index .

ENV RUST_LOG=info

# set the startup command to run your binary
CMD ["./telegram-index"]