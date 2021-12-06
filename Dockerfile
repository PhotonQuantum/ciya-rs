FROM rust:slim-bullseye AS builder

WORKDIR /work

RUN apt-get update && apt-get -y install llvm libclang-dev libopencv-dev make wget clang && rm -rf /var/lib/apt/lists/*

COPY src ./src

COPY resources ./resources

COPY Cargo.toml Cargo.lock ./

COPY Makefile ./

RUN make bot

RUN make copy-ort

FROM debian:bullseye-slim

WORKDIR /work

RUN apt-get update && apt-get -y install libopencv-core4.5 libopencv-objdetect4.5 && rm -rf /var/lib/apt/lists/*

COPY --from=builder ./work/dist/ciya_bot ./

COPY --from=builder ./work/dist/libs/* /usr/lib/

EXPOSE 8080

CMD ["./ciya_bot"]
