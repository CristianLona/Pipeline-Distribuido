# Etapa 1: Builder multi-stage
FROM rust:slim-bullseye AS builder
WORKDIR /usr/src/app
RUN apt-get update && apt-get install -y pkg-config libssl-dev

# Copiamos todo el workspace
COPY . .

# Compilamos especificando solo el binario del edge
RUN cd rust && cargo build --release --bin edge

# Etapa 2: Imagen mínima para producción
FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app

# Copiamos solo el ejecutable compilado
COPY --from=builder /usr/src/app/rust/target/release/edge /app/edge

# Exponemos el puerto de Axum
EXPOSE 8080

# Ejecutamos el binario
CMD ["./edge"]
