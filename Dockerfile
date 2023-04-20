FROM rust:1.67 as builder
WORKDIR /app
COPY . .
RUN cargo install --path .

FROM debian:bullseye-slim
RUN apt-get update
COPY --from=builder /app /app
ENTRYPOINT [ "/rana" ]
CMD [ "" ]
