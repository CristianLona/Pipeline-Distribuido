#!/bin/bash
# latencia_iot.sh
# Aplica un delay de 80ms con un jitter de 20ms para simular una red IoT inestable.

INTERFACE=${1:-zt0}

echo "Configurando latencia IoT en la interfaz: $INTERFACE"

# Primero limpiamos por si había otra regla activa
sudo tc qdisc del dev $INTERFACE root 2>/dev/null

# Aplicamos la nueva regla
sudo tc qdisc add dev $INTERFACE root netem delay 80ms 20ms

echo "Escenario activado: Delay 80ms + Jitter 20ms."
