#!/bin/bash
# baseline.sh
# Elimina todas las reglas tc activas y restaura la red limpia.

INTERFACE=${1:-zt0}

echo "Limpiando reglas de red en la interfaz: $INTERFACE"

# Ejecutamos el borrado ignorando errores si no había reglas previas
sudo tc qdisc del dev $INTERFACE root 2>/dev/null

echo "Red restaurada a su estado base (sin degradación)."
