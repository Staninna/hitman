FROM rust:1-bookworm AS builder

WORKDIR /usr/src/app

COPY Cargo.toml Cargo.lock ./
RUN mkdir src &&                                                               \
	echo "fn main() {}" > src/main.rs &&                                       \
	cargo build --release

COPY src ./src
COPY migrations ./migrations
COPY templates ./templates
COPY static ./static

RUN rm -f target/release/deps/hitman* && cargo build --release

FROM debian:bookworm-slim

RUN groupadd --system app && useradd --system --gid app app

RUN mkdir -p /app/data && chown -R app:app /app

WORKDIR /app

COPY --from=builder /usr/src/app/target/release/hitman .
COPY --from=builder /usr/src/app/migrations ./migrations
COPY --from=builder /usr/src/app/templates ./templates
COPY --from=builder /usr/src/app/static ./static

RUN chown -R app:app .

USER app

ENV DATABASE_URL="sqlite:/app/data/hitman.db"

EXPOSE 3000

CMD ["./hitman"]