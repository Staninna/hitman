# ---- Builder Stage ----
# Use a specific version of the Rust image based on Debian Bookworm.
# This ensures that the build environment is consistent and compatible with the
# runtime environment.
FROM rust:1-bookworm AS builder

WORKDIR /usr/src/app

# Copy manifests and pre-build dependencies to leverage Docker layer caching.
# This step builds only the dependencies, and it's cached as long as
# Cargo.toml and Cargo.lock don't change.
COPY Cargo.toml Cargo.lock ./
RUN mkdir src &&                                                               \
	echo "fn main() {}" > src/main.rs &&                                       \
	cargo build --release

# Now, copy the rest of the application source code.
# This layer will be invalidated if any of these files change.
COPY src ./src
COPY migrations ./migrations
COPY templates ./templates
COPY static ./static
COPY .sqlx ./.sqlx

# We need to set this to true to avoid the sqlx-cli trying to connect to the database.
ENV SQLX_OFFLINE=true

# Build the actual application.
# We remove the dummy binary first to ensure a clean build of our app.
RUN rm -f target/release/deps/hitman* && cargo build --release

# ---- Final/Runtime Stage ----
# Use a small, secure base image from the same OS family as the builder.
FROM debian:bookworm-slim

# Create a non-root user and group for security purposes.
# Running as a non-root user is a container security best practice.
RUN groupadd --system app && useradd --system --gid app app

# Create a directory for the app and a separate one for the database.
# This makes it easier to manage permissions and mount volumes for persistent
# data.
RUN mkdir -p /app && chown -R app:app /app

WORKDIR /app

# Copy the compiled binary and other necessary assets from the builder stage.
COPY --from=builder /usr/src/app/target/release/hitman .
COPY --from=builder /usr/src/app/migrations ./migrations
COPY --from=builder /usr/src/app/templates ./templates
COPY --from=builder /usr/src/app/static ./static

# Ensure the app user owns all the application files.
RUN chown -R app:app .

# Switch to the non-root user.
USER app

# Expose the port the app listens on for documentation and tooling.
EXPOSE 3000

# Set the startup command to run the binary.
CMD ["./hitman"] 