#!/bin/bash

# Detener el script inmediatamente si algún comando falla
set -e

echo "========================================"
echo "Iniciando construcción de imágenes..."
echo "========================================"

# 1. Construir el Coordinador
echo "[1/3] Construyendo app-coordinador:v1..."
docker build -t app-coordinador:v1 -f docker/coordinador/Dockerfile .

# 2. Construir el Edge
echo "[2/3] Construyendo app-edge:v1..."
docker build -t app-edge:v1 -f docker/edge/Dockerfile .

# 3. Construir los Sensores
echo "[3/3] Construyendo app-sensor:v1..."
docker build -t app-sensor:v1 -f docker/sensor/Dockerfile .

echo "¡Todas las imágenes se construyeron correctamente!"
echo "========================================"

# 4. Empaquetar para k3s
echo "Empaquetando las imágenes en 'imagenes-rust.tar' para k3s..."
docker save app-coordinador:v1 app-edge:v1 app-sensor:v1 -o imagenes-rust.tar

echo "¡Proceso completado!"
echo ""
echo "Siguiente paso:"
echo "Copia el archivo 'imagenes-rust.tar' a tus nodos (si es necesario) y ejecuta:"
echo "  sudo k3s ctr images import imagenes-rust.tar"