FROM registry.suse.com/bci/rust:latest AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

FROM registry.suse.com/bci/bci-micro:latest

ARG REVISION=unknown
LABEL org.opencontainers.image.revision=$REVISION

COPY --from=builder /app/target/release/pack /usr/local/bin/pack

RUN chmod 1777 /tmp

ENTRYPOINT ["/usr/local/bin/pack"]
