#!/bin/bash
# perdida_paquetes.sh
# Aplica una pérdida de paquetes del 8% sobre la interfaz.

INTERFACE=${1:-zt0}

echo "Configurando pérdida de paquetes en la interfaz: $INTERFACE"

# Limpiamos reglas previas
sudo tc qdisc del dev $INTERFACE root 2>/dev/null

# Aplicamos la regla
sudo tc qdisc add dev $INTERFACE root netem loss 8%

echo "Escenario activado: Pérdida de paquetes 8%."
