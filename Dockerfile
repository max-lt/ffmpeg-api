# ffmpeg-api image
FROM rust:1.73-alpine3.18 as builder

RUN apk add --no-cache libc-dev

RUN mkdir -p /build/ffmpeg-api

WORKDIR /build/ffmpeg-api

COPY . /build/ffmpeg-api

RUN cargo build --release

FROM alpine:3.18

RUN apk add --no-cache ffmpeg

COPY --from=builder /build/ffmpeg-api/target/release/ffmpeg-api /usr/local/bin/ffmpeg-api

ENTRYPOINT ["/usr/local/bin/ffmpeg-api"]

EXPOSE 8080
