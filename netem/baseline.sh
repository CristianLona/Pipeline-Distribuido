#!/bin/bash
# baseline.sh
# Elimina todas las reglas tc activas y restaura red limpia.

IFACE=${1:-eth0} 
echo "Restaurando baseline en la interfaz $IFACE"
sudo tc qdisc del dev $IFACE root 2>/dev/null
echo "Baseline restaurado. La red ahora opera sin degradaciones."
