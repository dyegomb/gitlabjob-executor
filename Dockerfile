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
# FROM alpine:3.16 AS builder

# RUN apk add --no-cache \
#         ca-certificates \
#         gcc \
#         pkgconfig \
#         openssl-dev \
#         alpine-sdk

# ENV RUSTUP_HOME=/usr/local/rustup \
#     CARGO_HOME=/usr/local/cargo \
#     PATH=/usr/local/cargo/bin:$PATH \
#     RUST_VERSION=1.69.0

# RUN set -eux; \
#     apkArch="$(apk --print-arch)"; \
#     case "$apkArch" in \
#         x86_64) rustArch='x86_64-unknown-linux-musl'; rustupSha256='7aa9e2a380a9958fc1fc426a3323209b2c86181c6816640979580f62ff7d48d4' ;; \
#         aarch64) rustArch='aarch64-unknown-linux-musl'; rustupSha256='b1962dfc18e1fd47d01341e6897cace67cddfabf547ef394e8883939bd6e002e' ;; \
#         *) echo >&2 "unsupported architecture: $apkArch"; exit 1 ;; \
#     esac; \
#     url="https://static.rust-lang.org/rustup/archive/1.26.0/${rustArch}/rustup-init"; \
#     wget "$url"; \
#     echo "${rustupSha256} *rustup-init" | sha256sum -c -; \
#     chmod +x rustup-init; \
#     ./rustup-init -y --no-modify-path --profile minimal --default-toolchain $RUST_VERSION --default-host ${rustArch}; \
#     rm rustup-init; \
#     chmod -R a+w $RUSTUP_HOME $CARGO_HOME; \
#     rustup --version; \
#     cargo --version; \
#     rustc --version;

FROM debian:11 AS builder

RUN apt-get update \
    &&  apt-get install -y \
        build-essential \
        libssl-dev \
        pkg-config \
        wget         libssl-dev \
        pkg-config \
        librust-openssl-dev \
        musl-tools \
        musl-dev \\
        # librust-openssl-dev \
        musl-tools \
        musl-dev


ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH \
    RUST_VERSION=1.69.0

RUN set -eux; \
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
    rustc --version;

# FROM rust:latest AS builder

RUN rustup target add x86_64-unknown-linux-musl
# RUN apt update && apt install -y musl-tools musl-dev libssl-dev
RUN update-ca-certificates


WORKDIR /opt
COPY src /opt/src
COPY Cargo.toml /opt/

# RUN cargo build --release
RUN cargo build --target x86_64-unknown-linux-musl --release

# ##############

# # FROM debian:bullseye-slim
# FROM alpine
# FROM busybox
# FROM alpine:3.16
# FROM alpine:3.17
FROM scratch

# # RUN mkdir /opt
WORKDIR /opt

# COPY --from=builder /opt/target/release/gitlabjob /opt/
# COPY target/x86_64-unknown-linux-musl/release/gitlabjob /opt/
COPY --from=builder /opt/target/x86_64-unknown-linux-musl/release/gitlabjob /opt/
COPY .env /opt/



# # # RUN apt-get update; \
# # #     apt-get install -y --no-install-recommends \
# # #         ca-certificates \
# # #     && apt-get remove -y --auto-remove \
# # #     && rm -rf /var/lib/apt/lists/*

# RUN apk --no-cache add ca-certificates

CMD [ "/opt/gitlabjob" ]