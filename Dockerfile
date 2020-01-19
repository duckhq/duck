# Build server
FROM ekidd/rust-musl-builder:stable AS server-builder
ADD . ./
RUN sudo chown -R rust:rust .
RUN cargo build --release --target x86_64-unknown-linux-musl --features docker

# Build UI
FROM node:lts-alpine as ui-builder
WORKDIR /app
ENV VUE_APP_MY_DUCK_SERVER=
COPY ./ui/package*.json ./
RUN npm install
COPY ./ui .
RUN npm run build

# Copy to Alpine container
FROM alpine:latest
EXPOSE 15825
RUN apk --no-cache add ca-certificates
COPY --from=server-builder  /home/rust/src/target/x86_64-unknown-linux-musl/release/duck /usr/local/bin/
COPY --from=ui-builder /app/dist /usr/local/bin/ui
WORKDIR /usr/local/bin
ENTRYPOINT ["duck"]
CMD ["--help"]