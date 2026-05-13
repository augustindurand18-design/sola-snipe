#!/bin/bash
# VPS Ubuntu 24.04 Network Optimization (TCP BBR)

if [ "$EUID" -ne 0 ]; then
  echo "Veuillez exécuter ce script en tant que root (sudo)."
  exit 1
fi

echo "Activation de l'algorithme TCP BBR..."

cat <<EOF >> /etc/sysctl.conf
net.core.default_qdisc=fq
net.ipv4.tcp_congestion_control=bbr
net.ipv4.tcp_fastopen=3
net.ipv4.tcp_low_latency=1
EOF

sysctl -p

echo "Vérification de l'activation de BBR :"
lsmod | grep bbr || echo "BBR n'est pas chargé dans le noyau. Un redémarrage peut être nécessaire."
