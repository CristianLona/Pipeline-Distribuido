#!/bin/bash
# latencia_iot.sh
# Aplica delay 80ms jitter 20ms sobre la interfaz seleccionada.

IFACE=${1:-eth0}
echo "Aplicando latencia IoT (delay 80ms, jitter 20ms) en $IFACE"
# sudo tc qdisc add dev $IFACE root netem delay 80ms 20ms distribution normal
