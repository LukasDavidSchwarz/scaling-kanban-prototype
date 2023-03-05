FROM rust:1.67-buster AS base

ENV ENV=development
ENV CARGO_TARGET_DIR=../target

WORKDIR /src
RUN cargo init
RUN cargo fetch

FROM base AS development-watch

RUN cargo install cargo-watch
CMD [ "cargo-watch", "-x", "run"]

FROM base AS development

CMD [ "cargo", "run", "--offline"]

FROM base AS production

ENV ENV=production

CMD [ "cargo", "run", "--offline", "--release"]
