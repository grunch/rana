FROM rust:1.76 as builder
WORKDIR /app
COPY . .
RUN cargo install --path .

FROM debian:bookworm-slim
RUN apt-get update
RUN apt-get -y install libc6
COPY --from=builder /app /app
ENTRYPOINT [ "/rana" ]
CMD [ "" ]
