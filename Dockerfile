# Dockerfile optimizado para producción en GCP (multi-stage build)

# Paso 1: Compilación
FROM rust:1.80-slim-bookworm AS builder

WORKDIR /usr/src/app

# Instalar dependencias del sistema necesarias
RUN apt-get update && apt-get install -y pkg-config libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

# Copiar manifiesto y compilar dependencias para cachear capas de Docker
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Copiar código fuente real
COPY src ./src

# Forzar recompilación con el código real
RUN touch src/main.rs && cargo build --release

# Paso 2: Imagen de ejecución (ultra-liviana y segura)
FROM debian:bookworm-slim

WORKDIR /app

# Instalar certificados CA para peticiones HTTP salientes seguras (ej. API de Fivetran)
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

# Copiar el binario compilado desde el builder
COPY --from=builder /usr/src/app/target/release/mcp-sql-sentinel /app/mcp-sql-sentinel

# Copiar la configuración del WMS
COPY config.json /app/config.json

# Exponer el puerto por defecto de SSE
EXPOSE 3000

# Variables de entorno por defecto para producción
ENV PORT=3000
ENV TRANSPORT=sse

# Comando de inicio en modo SSE para Cloud Run
CMD ["/app/mcp-sql-sentinel", "--transport", "sse"]
