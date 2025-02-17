###############################################################################
## Client builder
###############################################################################
FROM node:21-slim AS client-builder
ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"
RUN corepack enable
WORKDIR /client-builder
COPY ./front .
RUN --mount=type=cache,id=pnpm,target=/pnpm/store pnpm install --frozen-lockfile
RUN pnpm run build

###############################################################################
## Server builder
###############################################################################
FROM rust:alpine3.21 AS server-builder
RUN apk add --update --no-cache \
            autoconf \
            gcc \
            gdb \
            git \
            libdrm-dev \
            libepoxy-dev \
            make \
            mesa-dev \
            strace \
            openssl \
            openssl-dev \
            musl-dev && \
    rm -rf /var/cache/apk && \
    rm -rf /var/lib/app/lists

WORKDIR /server-builder
COPY ./back .
RUN cargo build --release --locked

###############################################################################
## Final image
###############################################################################
FROM alpine:3.21

ENV USER=app \
    UID=1000

RUN apk add --update --no-cache \
            tzdata~=2025 \
            sqlite~=3.48 && \
    rm -rf /var/cache/apk && \
    rm -rf /var/lib/app/lists && \
    mkdir -p /app/db && \
    mkdir -p /app/rss
# Copy our build
COPY --from=server-builder /server-builder/target/release/podmixer /app
COPY --from=client-builder /client-builder/dist/ /app/static/
COPY ./back/migrations /app/migrations/

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
EXPOSE 3000

CMD [ "/app/podmixer" ]

