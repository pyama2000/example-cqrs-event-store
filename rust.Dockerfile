FROM rust:1.85.0 AS builder
ARG APPLICATION_NAME
ARG SERVICE_DIRECTORY
ENV WORKDIR=/usr/src/app/$APPLICATION_NAME
ENV CARGO_HOME=/usr/local/cargo/$APPLICATION_NAME
WORKDIR $WORKDIR
RUN apt-get update \
 && apt-get install --no-install-recommends -y protobuf-compiler="3.21.12-3" \
 && apt-get clean \
 && rm -rf /var/lib/apt/lists/*
COPY crates ../../crates
COPY proto ../../proto
RUN --mount=type=bind,source="${SERVICE_DIRECTORY}/src",target=src \
    --mount=type=bind,source="${SERVICE_DIRECTORY}/internal",target=internal \
    --mount=type=bind,source="${SERVICE_DIRECTORY}/Cargo.toml",target=Cargo.toml \
    --mount=type=bind,source="${SERVICE_DIRECTORY}/Cargo.lock",target=Cargo.lock \
    --mount=type=cache,target=$WORKDIR \
    --mount=type=cache,target="/usr/local/cargo/${APPLICATION_NAME}/registry/" \
    cargo build --locked --release \
 && cp "./target/release/${APPLICATION_NAME}" "/bin/application"

FROM gcr.io/distroless/cc-debian12:nonroot
COPY --from=builder "/bin/application" /application
USER nonroot
CMD ["/application"]
