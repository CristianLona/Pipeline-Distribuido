#!/bin/bash
# perdida_paquetes.sh
# Aplica loss 8% sobre la interfaz seleccionada.

IFACE=${1:-eth0}
echo "Aplicando perdida de paquetes (loss 8%) en $IFACE"
# sudo tc qdisc add dev $IFACE root netem loss 8%
