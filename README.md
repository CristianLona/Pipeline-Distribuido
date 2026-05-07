# Pipeline Distribuido IoT/Edge (Guía de Reproducción)

Este proyecto implementa un sistema distribuido de monitoreo de datos con tolerancia a fallos, utilizando **K3s** para la orquestación y **Rust** para la lógica de alta eficiencia.

## 1. Arquitectura del Sistema
El sistema se divide en tres capas que se comunican a través de una red virtual segura:
* **Sensor:** Genera datos y envía *heartbeats*.
* **Edge:** Procesa datos localmente y reintenta envíos si falla la red (Backoff exponencial).
* **Coordinador:** Monitorea la salud de los nodos y centraliza la información.

---

## 2. Requisitos de Software
Asegúrate de tener instaladas las siguientes versiones (o superiores):
* **Rust:** 1.75+ (`cargo`, `rustc`)
* **K3s:** v1.35.4+k3s1 (o versión estable reciente)
* **ZeroTier One:** Para la red virtual distribuida.
* **Docker & Docker Compose:** Para la construcción de imágenes.
* **Linux (Ubuntu 24.04 LTS recomendado):** Necesario para usar `tc netem`.

---

## 3. Configuración de la VPN (ZeroTier)
Antes de levantar el sistema, todos los nodos deben estar en la misma red virtual.

1.  **Unirse a la red:**
    ```bash
    sudo zerotier-cli join [ID_DE_TU_RED_ZEROTIER]
    ```
2.  **Identificar tu IP virtual y nombre de interfaz:**
    ```bash
    ip a | grep zt  # Busca la IP asignada (ej. 10.x.x.x) y la interfaz (ej. ztktivcv7u)
    ```
3.  **Identificar roles:** Define qué nodo actuará como **Master** y cuál como **Worker**, y anota sus respectivas IPs virtuales.

---

## 4. Instalación del Clúster (K3s)

### Paso 1: Configurar el Master (Nodo Principal)
En el nodo elegido como Master, ejecuta el siguiente comando para instalar K3s indicando su IP en la red de ZeroTier:
```bash
curl -sfL https://get.k3s.io | INSTALL_K3S_EXEC="--node-ip=[IP_MASTER_ZEROTIER] --flannel-iface=[INTERFAZ_ZEROTIER]" sh -
```

* **Obtén el Token:** Lo necesitarás para unir los workers.
  ```bash
  sudo cat /var/lib/rancher/k3s/server/node-token
  ```
* **Etiqueta los nodos:**
  ```bash
  sudo kubectl label nodes [NOMBRE_NODO_MASTER] vincular-edge=si
  sudo kubectl label nodes [NOMBRE_NODO_WORKER] vincular-edge=si
  ```

### Paso 2: Unir el Worker
En los nodos secundarios (Workers), ejecuta la instalación de K3s apuntando al Master:
```bash
curl -sfL https://get.k3s.io | K3S_URL=https://[IP_MASTER_ZEROTIER]:6443 \
K3S_TOKEN=[TU_TOKEN_AQUI] \
INSTALL_K3S_EXEC="--node-ip=[IP_WORKER_ZEROTIER] --flannel-iface=[INTERFAZ_ZEROTIER]" sh -
```

---

## 5. Construcción de Imágenes Docker
Si realizaste cambios en el código de Rust, debes reconstruir las imágenes:

```bash
# Construcción manual por rol
docker build -f docker/coordinador/Dockerfile -t pipeline-coordinador:latest .
docker build -f docker/edge/Dockerfile -t pipeline-edge:latest .
docker build -f docker/sensor/Dockerfile -t pipeline-sensor:latest .

# O usa el script automático:
./build-images.sh
```

---

## 6. Despliegue del Sistema

### Opción A: Kubernetes (Recomendado para producción)
```bash
# Aplica el manifiesto correspondiente a tu clúster (ej. k3s.yaml)
sudo kubectl apply -f k3s.yaml
```

### Opción B: Docker Compose (Para pruebas locales)
```bash
# Variables de entorno configuradas en docker-compose.yml
docker-compose up -d
```

---

## 7. Ejecución Nativa en Rust (Desarrollo)
Si prefieres correrlo sin contenedores para depurar:

1.  **Compilar:** `cargo build --release`
2.  **Ejecutar Coordinador:** `cargo run --bin coordinador`
3.  **Ejecutar Edge:** `COORDINATOR_URL="http://[IP_MASTER_ZEROTIER]:3001" cargo run --bin edge`

---

## 8. Simulación de Fallas (`tc netem`)
Usa estos scripts para probar la tolerancia a fallos. **Requieren sudo.**

* **Simular Latencia:** `sudo ./netem/latencia_iot.sh`
* **Simular Pérdida de Paquetes:** `sudo ./netem/perdida_paquetes.sh`
* **Simular Caída de Nodo:** `sudo ./netem/falla_nodo.sh`
* **Limpiar Reglas (Regresar a la normalidad):** `sudo ./netem/baseline.sh`

---

## 9. Notas y Limitaciones
* **Firewall:** Asegúrate de abrir el puerto `10250` para logs y el `6443` para el API de K3s.
* **ZeroTier:** La estabilidad de la VPN depende de la conexión de internet física; latencias mayores a 500ms pueden causar estados `NotReady` en los nodos.
* **Persistencia:** Actualmente los datos se procesan en tiempo real; si el Coordinador se reinicia, el historial de métricas en memoria se limpia.