#!/bin/sh

set -e

for f in "$@"; do
  jq -r '.[]._source.layers | .usb."usb.src", .usb."usb.dst", ."usb.capdata", ""' <"$f"
done | while read src; read dst; read pl;read x; do
  if [ "$pl" = "null" ]; then
    continue
  elif [ "$src" = "host" ]; then
    echo "> $pl"
  elif [ "$dst" = "host" ]; then
    echo "< $pl"
  fi
done


#./replay-prepare.sh gt-120b-kvm-sesson-20250529.json >gt-120b-kvm-sesson-20250529.json.txt
#./replay-prepare.sh gt-120b-kvm-sesson-20250603.json >gt-120b-kvm-sesson-20250603.json.txt
