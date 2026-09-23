#!/usr/bin/env bash
# Local Warpgate with template targets, for development and the README demo.
set -euo pipefail

cd "$(dirname "$0")/.."

WARPGATE_URL=https://localhost:8888
WARPGATE_ADMIN_PASSWORD=${WARPGATE_ADMIN_PASSWORD:-admin}
export WARPGATE_ADMIN_PASSWORD
CONFIG_PATH=data/config.toml
VHS_IMAGE=ghcr.io/charmbracelet/vhs:v0.10.0

usage() {
    echo "Usage: $0 [up|down|reset|demo]"
}

api() {
    local method=$1 path=$2 body=${3-}
    local response status
    response=$(curl -ksS -b "$cookie_jar" -c "$cookie_jar" -X "$method" \
        -H 'Content-Type: application/json' ${body:+--data "$body"} \
        -w '\n%{http_code}' "$WARPGATE_URL$path")
    status=${response##*$'\n'}
    response=${response%$'\n'*}
    if [[ $status != 2* ]]; then
        echo "$method $path failed with HTTP $status: $response" >&2
        echo "A half-seeded instance can be cleared with '$0 reset'." >&2
        exit 1
    fi
    printf '%s' "$response"
}

seed() {
    cookie_jar=$(mktemp)
    trap 'rm -f "$cookie_jar"' EXIT

    api POST /@warpgate/api/auth/login \
        "$(jq -n --arg password "$WARPGATE_ADMIN_PASSWORD" '{username: "admin", password: $password}')" >/dev/null

    local role_id admin_id
    role_id=$(api POST /@warpgate/admin/api/roles '{"name": "demo", "description": "Template targets"}' | jq -r .id)
    admin_id=$(api GET /@warpgate/admin/api/users | jq -r '.[] | select(.username == "admin") | .id')
    api POST "/@warpgate/admin/api/users/$admin_id/roles/$role_id" '{}' >/dev/null

    # Payloads are built into variables first: a jq failure inside `< <(...)` would not stop the script.
    local groups group group_ids='{}' group_id
    groups=$(jq -c '.groups[]' dev/targets.json)
    while read -r group; do
        group_id=$(api POST /@warpgate/admin/api/target-groups "$group" | jq -r .id)
        group_ids=$(jq -c --arg name "$(jq -r .name <<<"$group")" --arg id "$group_id" '. + {($name): $id}' <<<"$group_ids")
    done <<<"$groups"

    local targets target target_id
    targets=$(jq -c --argjson group_ids "$group_ids" '
        def common: {
            name, description, group_id: $group_ids[.group // ""],
            ticket_requests_disabled: true, ticket_require_approval: false, require_approval: false
        } | del(.. | nulls);
        (.targets[] | common + {options: {
            kind: "Ssh", host, port, username, allow_insecure_algos: false,
            auth: {kind: "Password", password}
        }}),
        (.http_targets[] | common + {options: {
            kind: "Http", url, headers: {}, tls: {mode: "Disabled", verify: false}
        }})
    ' dev/targets.json)
    while read -r target; do
        target_id=$(api POST /@warpgate/admin/api/targets "$target" | jq -r .id)
        api POST "/@warpgate/admin/api/targets/$target_id/roles/$role_id" >/dev/null
    done <<<"$targets"

    local token
    token=$(api POST /@warpgate/api/profile/api-tokens \
        "$(jq -n '{label: "warpgate-connect-dev", expiry: (now + 365 * 86400 | todate)}')" | jq -r .secret)

    (
        umask 077
        cat >"$CONFIG_PATH" <<EOF
warpgate_api_url = "$WARPGATE_URL/@warpgate/api/targets"
warpgate_token = "$token"
warpgate_username = "admin"
warpgate_port = 2222
EOF
    )
    echo "Seeded $(jq '.targets + .http_targets | length' dev/targets.json) targets, wrote $CONFIG_PATH"
}

up() {
    if ! docker compose run --rm --no-deps --entrypoint test warpgate -f /data/warpgate.yaml; then
        docker compose run --rm --no-deps warpgate unattended-setup \
            --data-path /data --http-port 8888 --ssh-port 2222 \
            --external-host localhost --host-key-verification auto-accept
    fi

    docker compose up -d --wait

    if [[ -f $CONFIG_PATH ]]; then
        echo "$CONFIG_PATH exists, skipping seeding"
    else
        seed
    fi

    cat <<EOF

Warpgate is up: $WARPGATE_URL (admin / $WARPGATE_ADMIN_PASSWORD)
Run the app:    cargo run -- --skip-update --config $CONFIG_PATH
Targets web-prod-01 and web-staging-01 connect for real; SSH asks for the Warpgate admin password.
EOF
}

reset() {
    read -rp "Delete the local Warpgate data and $CONFIG_PATH? [y/N] " answer
    [[ $answer == [yY] ]] || exit 1
    docker compose down --volumes
    rm -f "$CONFIG_PATH"
}

demo() {
    [[ -f $CONFIG_PATH ]] || up
    cargo build --release
    mkdir -p data/demo

    # Rootless Docker already maps container root to the caller; rootful needs --user or the output is root's.
    local user_args=()
    if ! docker info --format '{{.SecurityOptions}}' | grep -q rootless; then
        user_args=(--user "$(id -u):$(id -g)" -e HOME=/tmp)
    fi

    docker run --rm --network host "${user_args[@]}" -v "$PWD:/vhs" "$VHS_IMAGE" dev/demo.tape
    docker run --rm "${user_args[@]}" -v "$PWD:/vhs" -w /vhs --entrypoint ffmpeg "$VHS_IMAGE" \
        -loglevel error -y -i data/demo/targets.png -c:v libwebp -lossless 1 \
        shared/img/warpgate-connect.webp
    echo "Wrote shared/img/warpgate-connect.webp; the gif and the other screenshots are in data/demo/"
}

case ${1:-up} in
up) up ;;
down) docker compose down ;;
reset) reset ;;
demo) demo ;;
*)
    usage
    exit 1
    ;;
esac
