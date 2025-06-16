#!/bin/sh

set -e

for f in src/replay-120b-part*.json; do
  jq -r '.[]._source.layers | .usb."usb.src", .usb."usb.dst", ."usb.capdata", ""' <"$f"
done | while read src; read dst; read pl;read x; do
  if [ "$pl" = "null" ]; then
    continue
  elif [ "$src" = "host" ]; then
    echo "> $pl"
  elif [ "$dst" = "host" ]; then
    echo "< $pl"
  fi
done >src/replay-120b.txt