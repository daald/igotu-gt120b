#!/bin/sh

set -ex

for f in ../../usbmon-log/gt-120b/*.json.txt; do
  [ "$f" != "../../usbmon-log/gt-120b/somethingidontwant.json.txt" ] || continue  # this script is different (fw update)
  if ! ./cargo-run.sh  --bestreplay --sim-file-name "$f"; then
    echo "$f"
    break
  fi
done

echo "all good."
