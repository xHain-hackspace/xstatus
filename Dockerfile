FROM rust:1.78 as builder

ENV USER=xstatus
ENV UID=10001

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"

RUN cargo new --bin xstatus

WORKDIR /xstatus

# copy over your manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# this build step will cache your dependencies
RUN cargo build --release && rm src/*.rs

# copy the real sources
COPY ./src ./src

# build the binary
RUN rm ./target/release/deps/xstatus* && cargo build --release

FROM debian:bookworm-slim

COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

WORKDIR /xstatus

COPY --from=builder /xstatus/target/release/xstatus ./

USER xstatus:xstatus

CMD ["/xstatus/xstatus"]

