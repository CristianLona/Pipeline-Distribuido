#!/bin/bash
# falla_nodo.sh
# Simula la caída prolongada de un nodo Edge para forzar la alerta en el Coordinador.

# Buscar el nodo y el pod donde está corriendo el primer pod de edge
NODE_NAME=$(sudo kubectl get pods -l app=edge --field-selector status.phase=Running -o jsonpath="{.items[0].spec.nodeName}" 2>/dev/null)
POD_NAME=$(sudo kubectl get pods -l app=edge --field-selector status.phase=Running -o jsonpath="{.items[0].metadata.name}" 2>/dev/null)

if [ -z "$NODE_NAME" ] || [ -z "$POD_NAME" ]; then
    echo "Error: No se encontró ningún Pod activo para el nodo Edge."
    exit 1
fi

echo "Simulando falla crítica en el nodo: $NODE_NAME (Pod: $POD_NAME)..."
echo "Desconectando el nodo lógicamente por 20 segundos..."

# Quitar la etiqueta para que Kubernetes sepa que ya no debe correr ahí
sudo kubectl label nodes $NODE_NAME vincular-edge-

# Forzar la eliminación inmediata del pod para que deje de enviar heartbeats al instante
echo "Forzando terminación del pod..."
sudo kubectl delete pod $POD_NAME --force --grace-period=0

# Esperar para que el Coordinador note la ausencia (>10s)
for i in {1..20}; do
    echo -n "."
    sleep 1
done
echo ""

echo "Restaurando el nodo $NODE_NAME..."
# Devolver la etiqueta para que Kubernetes vuelva a crear el pod
sudo kubectl label nodes $NODE_NAME vincular-edge=si

echo "Falla inyectada y nodo restaurado. Revisa los logs del Coordinador."
