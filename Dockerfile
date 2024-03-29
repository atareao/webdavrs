###############################################################################
## Builder
###############################################################################
FROM rust:1.64 AS builder

LABEL maintainer="Lorenzo Carbonell <a.k.a. atareao> lorenzo.carbonell.cerezo@gmail.com"

ARG TARGET=x86_64-unknown-linux-musl
ENV RUST_MUSL_CROSS_TARGET=$TARGET
ENV OPENSSL_LIB_DIR="/usr/lib/x86_64-linux-gnu"
ENV OPENSSL_INCLUDE_DIR="/usr/include/openssl"

RUN rustup target add x86_64-unknown-linux-musl && \
    apt-get update && \
    apt-get install -y \
        --no-install-recommends\
        pkg-config \
        musl-tools \
        build-essential \
        cmake \
        musl-dev \
        pkg-config \
        libssl-dev \
        && \
    apt-get clean && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY ./ .

RUN cargo build  --target x86_64-unknown-linux-musl --release

###############################################################################
## Final image
###############################################################################
FROM alpine:3.17

RUN apk add --update --no-cache \
            su-exec~=0.2 &&\
    rm -rf /var/cache/apk && \
    rm -rf /var/lib/app/lists*
# Copy the user

# Set the work dir
WORKDIR /app

COPY entrypoint.sh /app/
COPY migrations/ /app/migrations/
# Copy our build
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/webdavrs /app/

ENTRYPOINT ["/bin/sh", "/app/entrypoint.sh"]
CMD ["/app/webdavrs"]
