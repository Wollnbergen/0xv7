#!/usr/bin/env bash
set -euo pipefail

NAME=scylla
VOLUME=scylla-data
IMAGE=scylladb/scylla:5.4

wait_for_cql() {
  echo "Waiting for cqlsh on $NAME..."
  for i in {1..90}; do
    if docker exec "$NAME" bash -lc 'cqlsh 127.0.0.1 -e "SHOW VERSION;"' >/dev/null 2>&1; then
      echo "cqlsh is ready"
      return 0
    fi
    sleep 2; printf "."
  done
  echo "Timed out waiting for cqlsh"
  docker logs --tail=200 "$NAME" || true
  return 1
}

run_container() {
  docker volume inspect "$VOLUME" >/dev/null 2>&1 || docker volume create "$VOLUME" >/dev/null
  docker run -d --name "$NAME" --hostname "$NAME" \
    -v "$VOLUME":/var/lib/scylla \
    -p 9042:9042 -p 7000:7000 -p 7001:7001 -p 7199:7199 -p 9100:9100 -p 9180:9180 \
    "$IMAGE" \
    --developer-mode 1 --overprovisioned 1 --smp 1 --memory 1G \
    --listen-address 0.0.0.0 --rpc-address 0.0.0.0 \
    --broadcast-address 127.0.0.1 --broadcast-rpc-address 127.0.0.1 \
    --seed-provider-parameters seeds=127.0.0.1 \
    --cluster-name dev-scylla
}

case "${1:-start}" in
  start)
    if docker inspect "$NAME" >/dev/null 2>&1; then
      if [ "$(docker inspect -f '{{.State.Running}}' "$NAME")" = "true" ]; then
        echo "Container: $NAME (running)"; exit 0
      fi
      docker start "$NAME"
    else
      run_container
    fi
    wait_for_cql
    ;;
  stop) docker stop "$NAME" ;;
  restart) docker restart "$NAME"; wait_for_cql ;;
  rm) docker rm -fv "$NAME" ;;
  recreate) docker rm -fv "$NAME" || true; run_container; wait_for_cql ;;
  reset)
    docker rm -fv "$NAME" || true
    docker volume rm -f "$VOLUME" || true
    run_container; wait_for_cql
    ;;
  status)
    if docker inspect "$NAME" >/dev/null 2>&1; then
      state="$(docker inspect -f '{{.State.Status}}' "$NAME")"
      echo "Container: $NAME ($state)"
      docker ps -a --filter "name=^/$NAME$"
    else
      echo "Container: $NAME (absent)"
    fi
    ;;
  logs) docker logs -f "$NAME" ;;
  cql)
    shift || true
    # interactive if TTY
    if [ -t 1 ]; then IT="-it"; else IT=""; fi
    docker exec -e CQLSH_HOST=127.0.0.1 $IT "$NAME" cqlsh "$@"
    ;;
  *)
    echo "Usage: $0 {start|status|recreate|reset|stop|restart|rm|logs|cql [args...]}"
    exit 1
    ;;
esac