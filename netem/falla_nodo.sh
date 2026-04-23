#!/bin/bash
# falla_nodo.sh
# Detiene un contenedor edge específico (argumento: nombre del contenedor).

CONTAINER_NAME=${1:-edge}
echo "Simulando falla de nodo: deteniendo contenedor $CONTAINER_NAME..."
# docker stop $CONTAINER_NAME
