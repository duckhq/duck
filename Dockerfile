# Build server
FROM ekidd/rust-musl-builder:stable AS server-builder
ARG VERSION=0.1.0
ADD . ./
RUN sudo chown -R rust:rust .
RUN sed -i -e "/^version/ s/[[:digit:]].[[:digit:]].[[:digit:]]/$VERSION/" Cargo.toml
RUN cargo build --release --target x86_64-unknown-linux-musl --features docker

# Build frontend
FROM node:lts-alpine as frontend-builder
ARG VERSION=0.1.0
WORKDIR /app
ENV VUE_APP_MY_DUCK_SERVER=
COPY ./web/package*.json ./
RUN npm install
COPY ./web .
RUN sed -i -e "/version/ s/[[:digit:]].[[:digit:]].[[:digit:]]/$VERSION/" package.json
RUN npm run build

# Copy to Alpine container
FROM alpine:latest
EXPOSE 15825
RUN apk --no-cache add ca-certificates
COPY --from=server-builder  /home/rust/src/target/x86_64-unknown-linux-musl/release/duck /usr/local/bin/
COPY --from=frontend-builder /app/dist /usr/local/bin/web
WORKDIR /usr/local/bin
ENTRYPOINT ["duck"]
CMD ["--help"]