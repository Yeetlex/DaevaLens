#!/bin/bash
# Desarrollo en Hyprland: permiso de captura, arranque y overlays al escritorio del juego.
set -e
bin="$(dirname "$0")/../src-tauri/target/debug/daevalens"
bin="$(realpath "$bin")"
pkill -x daevalens 2>/dev/null || true
getcap "$bin" | grep -q cap_net_raw || sudo ${SUDO_ASKPASS:+-A} setcap cap_net_raw=ep "$bin"
hyprctl dispatch "hl.dsp.exec_cmd(\"sh -c '$bin > /tmp/daevalens-dev.log 2>&1'\")" >/dev/null
sleep 4
ws=$(hyprctl -j clients | python3 -c 'import sys,json;print(next((c["workspace"]["name"] for c in json.load(sys.stdin) if c["class"].startswith("steam_app")),""))')
[[ -z $ws ]] && exit 0
hyprctl -j clients | python3 -c 'import sys,json;[print(c["address"]) for c in json.load(sys.stdin) if c["title"].startswith("DaevaLens ")]' |
while read -r addr; do
    hyprctl dispatch "hl.dsp.window.move({ window = \"address:$addr\", workspace = \"$ws\", follow = false })" >/dev/null
done
