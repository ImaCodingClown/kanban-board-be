FROM rust

WORKDIR /app
COPY . .

RUN apt-get update && apt-get install -y pkg-config libssl-dev
RUN cargo build --release

EXPOSE 8080

CMD ["./target/release/kanban_backend"]

