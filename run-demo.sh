docker compose --env-file .env --env-file .env.demo up -d --scale backend=2 "$*"
