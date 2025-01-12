# Build stage
FROM rust:latest as builder

# Créer le répertoire de travail
WORKDIR /usr/src/app

# Installer protoc
RUN apt-get update && apt-get install -y protobuf-compiler

# Copier les fichiers Cargo.toml et Cargo.lock avant le reste pour bénéficier du cache Docker
COPY Cargo.toml Cargo.lock ./

# Créer un projet vide pour que Docker puisse mettre en cache les dépendances
RUN mkdir src
RUN echo "fn main() {}" > src/main.rs

# Télécharger les dépendances sans construire le projet
RUN cargo build --release
RUN rm -rf src

# Copier tous les fichiers source
COPY . .

# Build en mode release
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Installer les dépendances nécessaires
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copier l'exécutable depuis le builder
COPY --from=builder /usr/src/app/target/release/embedder /usr/local/bin/embedder

# Définir l'exécutable comme point d'entrée
ENTRYPOINT ["embedder"]