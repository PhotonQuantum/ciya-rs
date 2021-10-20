FROM rust:slim-bullseye AS builder

WORKDIR /work

RUN apt-get -y update

RUN apt-get -y install libopencv-dev llvm libclang-dev

RUN apt-get -y install libssl-dev ca-certificates

RUN apt-get -y install make wget clang

COPY src ./src

COPY resources ./resources

COPY Cargo.toml Cargo.lock ./

COPY Makefile ./

RUN make bot

RUN make copy-ort

FROM debian:bullseye-slim

WORKDIR /work

RUN apt-get -y update

RUN apt-get -y install libopencv-core4.5 libopencv-objdetect4.5

RUN apt-get -y install ca-certificates

COPY --from=builder ./work/dist/ciya_bot ./

COPY --from=builder ./work/dist/libs/* /usr/lib/

CMD ["./ciya_bot"]
