#!/bin/bash
# status.sh
# Muestra el estado actual de tc qdisc en la interfaz especificada.

INTERFACE=${1:-zt0}

echo "Estado actual de las reglas de red (tc qdisc) en la interfaz: $INTERFACE"
echo "------------------------------------------------------------"

sudo tc qdisc show dev $INTERFACE

echo "------------------------------------------------------------"
