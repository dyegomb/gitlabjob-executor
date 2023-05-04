# # FROM debian:sid AS builder

# # USER root

# # WORKDIR /opt
# # COPY src /opt/src
# # COPY Cargo.toml /opt/

# # # RUN apt-get update \
# # #     && apt-get install -y \
# # #         cargo \
# # #         libssl-dev \
# # #         pkg-config

# # # RUN cargo update -p winnow@0.4.6 --precise 0.4.1 \
# # #     && cargo build --release

# # RUN apt-get update \
# #     && apt-get install -y \
# #         curl \
# #         build-essential \
# #         libssl-dev \
# #         pkg-config \
# #         librust-openssl-dev \
# #         musl-tools \
# #         musl-dev \
# #     && curl https://sh.rustup.rs -sSf -o /tmp/rustup.sh \
# #     && chmod +x /tmp/rustup.sh \
# #     && /tmp/rustup.sh -y

# # RUN /root/.cargo/bin/rustup target add x86_64-unknown-linux-musl 

# # RUN /root/.cargo/bin/cargo build --target x86_64-unknown-linux-musl --release

# # RUN mkdir -p /opt/fake_folder/etc/ssl/

# FROM alpine AS builder

# WORKDIR /opt
# COPY src /opt/src
# COPY Cargo.toml /opt/

# RUN apk update \
#     && apk add \
#         rustup \
#         pkgconf \
#         openssl-dev \
#         alpine-sdk \
#     && rustup-init -y
#     # && apk add curl \
#     # && curl https://sh.rustup.rs -sSf -o /tmp/rustup.sh \
#     # && chmod +x /tmp/rustup.sh \
#     # && /tmp/rustup.sh -y


# RUN /root/.cargo/bin/rustup target add x86_64-unknown-linux-gnu
# # RUN /root/.cargo/bin/cargo build --release --target x86_64-unknown-linux-gnu

# #########

# # FROM scratch
# # FROM alpine

# # # # COPY --from=builder /opt/fake_folder/etc /
# # # # COPY --from=builder /etc/ssl/certs /etc/ssl/certs
# # # # COPY --from=builder /etc/hosts /etc/
# # # # COPY --from=builder /etc/resolv.conf /etc/
# # # # COPY --from=builder /opt/target/x86_64-unknown-linux-musl/release/gitlabjob /gitlabjob
# # # # COPY .env /.env
# # # COPY --from=builder /opt/target/x86_64-unknown-linux-musl/release/gitlabjob /opt/gitlabjob
# # # COPY .env /opt/.env
# # COPY --from=builder /opt/target/x86_64-unknown-linux-gnu/release/gitlabjob /opt/gitlabjob
# # COPY .env /opt/.env

# # WORKDIR /opt
# # CMD [gitlabjob]
# # ENTRYPOINT [ "/gitlabjob" ]
# #rustup target add x86_64-unknown-linux-musl
# # curl https://sh.rustup.rs -sSf | sh -s -- -y
# # /root/.cargo/bin/rustup 
# #/root/.cargo/bin/rustup target add x86_64-unknown-linux-musl

# CMD [ "/opt/gitlabjob" ]


######## 

# # https://github.com/rust-lang/docker-rust/blob/master/Dockerfile-alpine.template
# https://github.com/rust-lang/docker-rust/blob/master/1.69.0/alpine3.17/Dockerfile

FROM debian:bullseye-slim AS builder

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH \
    RUST_VERSION=1.69.0

RUN set -eux; \
    apt-get update; \
    apt-get install -y --no-install-recommends \
        ca-certificates \
        gcc \
        libc6-dev \
        wget \
        pkg-config \
        libssl-dev \
        ; \
    dpkgArch="$(dpkg --print-architecture)"; \
    case "${dpkgArch##*-}" in \
        amd64) rustArch='x86_64-unknown-linux-gnu'; rustupSha256='0b2f6c8f85a3d02fde2efc0ced4657869d73fccfce59defb4e8d29233116e6db' ;; \
        armhf) rustArch='armv7-unknown-linux-gnueabihf'; rustupSha256='f21c44b01678c645d8fbba1e55e4180a01ac5af2d38bcbd14aa665e0d96ed69a' ;; \
        arm64) rustArch='aarch64-unknown-linux-gnu'; rustupSha256='673e336c81c65e6b16dcdede33f4cc9ed0f08bde1dbe7a935f113605292dc800' ;; \
        i386) rustArch='i686-unknown-linux-gnu'; rustupSha256='e7b0f47557c1afcd86939b118cbcf7fb95a5d1d917bdd355157b63ca00fc4333' ;; \
        *) echo >&2 "unsupported architecture: ${dpkgArch}"; exit 1 ;; \
    esac; \
    url="https://static.rust-lang.org/rustup/archive/1.26.0/${rustArch}/rustup-init"; \
    wget "$url"; \
    echo "${rustupSha256} *rustup-init" | sha256sum -c -; \
    chmod +x rustup-init; \
    ./rustup-init -y --no-modify-path --profile minimal --default-toolchain $RUST_VERSION --default-host ${rustArch}; \
    rm rustup-init; \
    chmod -R a+w $RUSTUP_HOME $CARGO_HOME; \
    rustup --version; \
    cargo --version; \
    rustc --version; \
    apt-get remove -y --auto-remove \
        wget \
        ; \
    rm -rf /var/lib/apt/lists/*;

WORKDIR /opt
COPY src /opt/src
COPY Cargo.toml /opt/

RUN cargo build --release

##############

# FROM debian:bullseye-slim
FROM alpine

# COPY --from=builder /opt/target/release/gitlabjob /opt/
COPY target/x86_64-unknown-linux-musl/release/gitlabjob /opt/
COPY .env /opt/

WORKDIR /opt

RUN apt-get update; \
    apt-get install -y --no-install-recommends \
        ca-certificates \
    && apt-get remove -y --auto-remove \
    && rm -rf /var/lib/apt/lists/*

# RUN apk --no-cache add ca-certificates

CMD [ "/opt/gitlabjob" ]