#!/bin/bash
# Script d'optimisation et d'installation VPS pour Solana HFT Sniper (Rust Edition)
set -e

echo "🚀 [1/4] Optimisation de la stack réseau Linux (HFT / Low Latency)..."

cat <<EOF | sudo tee /etc/sysctl.d/99-hft-solana.conf
# Optimisations TCP BBR
net.ipv4.tcp_fastopen = 3
net.core.default_qdisc = fq
net.ipv4.tcp_congestion_control = bbr

# Buffers réseau massifs pour supporter le flux gRPC (Yellowstone)
net.ipv4.tcp_rmem = 4096 87380 33554432
net.ipv4.tcp_wmem = 4096 65536 33554432
net.core.rmem_max = 33554432
net.core.wmem_max = 33554432

# Recyclage des sockets pour éviter l'épuisement des ports
net.ipv4.tcp_tw_reuse = 1
net.ipv4.tcp_fin_timeout = 15
EOF

sudo sysctl --system

echo "🦀 [2/4] Installation de l'environnement Rust..."
if ! command -v cargo &> /dev/null
then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
else
    echo "Rust est déjà installé, mise à jour..."
    rustup update
fi

echo "📦 [3/4] Installation des dépendances de compilation (gRPC, SSL)..."
sudo apt-get update
sudo apt-get install -y build-essential pkg-config libssl-dev protobuf-compiler clang

echo "⚙️ [4/4] Compilation du bot HFT (Mode Release avec LTO)..."
cd rust-bot
# Force la compilation avec un seul Codegen Unit pour une optimisation maximale
CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo build --release

echo "✅ Terminé !"
echo ""
echo "👉 Pour lancer le bot, allez dans le dossier rust-bot et tapez :"
echo "RUST_LOG=info ./target/release/solana-hft-sniper"
