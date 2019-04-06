FROM alpine

RUN mkdir -p /test

RUN apk update && apk upgrade && \
    apk add --no-cache gcc libc-dev rust cargo
