#!/usr/bin/env bash
# Genera CA + cert de coordinador + cert de edge para mTLS
# Uso: ./gen-certs.sh
set -euo pipefail

DIR="$(cd "$(dirname "$0")" && pwd)/certs"
mkdir -p "$DIR"
cd "$DIR"

echo ">>> Generando CA (Pipeline-CA)"
openssl genrsa -out ca.key 4096
openssl req -x509 -new -nodes -key ca.key -sha256 -days 3650 \
  -out ca.crt -subj "/CN=Pipeline-CA/O=Pipeline-Distribuido"

# ---------- Coordinador (servidor) ----------
echo ">>> Generando cert para coordinador"
cat > coordinador.cnf <<'EOF'
[req]
distinguished_name = req
req_extensions = v3_req
prompt = no
[req]
CN = coordinador
[v3_req]
keyUsage = critical, digitalSignature, keyEncipherment
extendedKeyUsage = serverAuth, clientAuth
subjectAltName = @alt_names
[alt_names]
DNS.1 = coordinador-service
DNS.2 = coordinador-service.default
DNS.3 = coordinador-service.default.svc.cluster.local
DNS.4 = coordinador
DNS.5 = localhost
IP.1 = 127.0.0.1
EOF

openssl genrsa -out coordinador.key 2048
openssl req -new -key coordinador.key -out coordinador.csr \
  -subj "/CN=coordinador" -config coordinador.cnf
openssl x509 -req -in coordinador.csr -CA ca.crt -CAkey ca.key \
  -CAcreateserial -out coordinador.crt -days 825 -sha256 \
  -extensions v3_req -extfile coordinador.cnf

# ---------- Edge (cliente) ----------
echo ">>> Generando cert para edge"
cat > edge.cnf <<'EOF'
[req]
distinguished_name = req
req_extensions = v3_req
prompt = no
[req]
CN = edge
[v3_req]
keyUsage = critical, digitalSignature, keyEncipherment
extendedKeyUsage = clientAuth, serverAuth
subjectAltName = @alt_names
[alt_names]
DNS.1 = edge-service
DNS.2 = edge
DNS.3 = localhost
IP.1 = 127.0.0.1
EOF

openssl genrsa -out edge.key 2048
openssl req -new -key edge.key -out edge.csr \
  -subj "/CN=edge" -config edge.cnf
openssl x509 -req -in edge.csr -CA ca.crt -CAkey ca.key \
  -CAcreateserial -out edge.crt -days 825 -sha256 \
  -extensions v3_req -extfile edge.cnf

# Limpieza
rm -f *.csr *.cnf *.srl

echo
echo ">>> Certs generados en: $DIR"
ls -la "$DIR"
