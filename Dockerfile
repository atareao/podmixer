###############################################################################
## Builder
###############################################################################
FROM rust:1.74 AS builder

LABEL maintainer="Lorenzo Carbonell <a.k.a. atareao> lorenzo.carbonell.cerezo@gmail.com"

ARG TARGET=x86_64-unknown-linux-musl
ENV RUST_MUSL_CROSS_TARGET=$TARGET \
    OPENSSL_LIB_DIR="/usr/lib/x86_64-linux-gnu" \
    OPENSSL_INCLUDE_DIR="/usr/include/openssl"

RUN rustup target add $TARGET && \
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

COPY Cargo.toml Cargo.lock ./
COPY src src

RUN cargo build --release --target $TARGET && \
    cp /app/target/$TARGET/release/podmixer /app/podmixer

###############################################################################
## Final image
###############################################################################
FROM alpine:3.18

ENV USER=app \
    UID=1000

RUN apk add --update --no-cache \
            tzdata~=2023c \
            sqlite~=3.41 && \
    rm -rf /var/cache/apk && \
    rm -rf /var/lib/app/lists && \
    mkdir -p /app/db

# Copy our build
COPY --from=builder /app/podmixer /app/
COPY ./assets /app/assets/
COPY ./migrations /app/migrations/
COPY ./rss /app/rss/
COPY ./templates /app/templates/

# Create the user
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/${USER}" \
    --shell "/sbin/nologin" \
    --uid "${UID}" \
    "${USER}" && \
    chown -R app:app /app

WORKDIR /app
USER app

CMD ["/app/podmixer"]
