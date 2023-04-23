FROM rust:1.68-buster AS base

ENV CARGO_TARGET_DIR=../backend_build
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse

WORKDIR /backend
COPY . .

FROM base AS development

RUN cargo build
CMD [ "cargo", "run"]

FROM development AS development-watch

RUN cargo install cargo-watch
CMD [ "cargo-watch", "-x", "run"]

FROM base AS production

RUN cargo build --release
CMD [ "cargo", "run", "--release"]
