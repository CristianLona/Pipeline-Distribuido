#!/bin/bash
# enlace_limitado.sh
# Aplica una limitación de ancho de banda y latencia simulando un enlace muy pobre.

INTERFACE=${1:-zt0}

echo "Configurando enlace limitado en la interfaz: $INTERFACE"

# Limpiamos reglas previas
sudo tc qdisc del dev $INTERFACE root 2>/dev/null

# Aplicamos la regla
sudo tc qdisc add dev $INTERFACE root netem rate 512kbit delay 50ms

echo "Escenario activado: Ancho de banda 512kbit + Delay 50ms."
