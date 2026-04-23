#!/bin/bash
# enlace_limitado.sh
# Aplica rate 512kbit delay 50ms sobre la interfaz seleccionada.

IFACE=${1:-eth0}
echo "Aplicando enlace limitado (rate 512kbit delay 50ms) en $IFACE"
# sudo tc qdisc add dev $IFACE root netem rate 512kbit delay 50ms
