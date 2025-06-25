#!/bin/sh

set -e

drop_in_lines=
for f in "$@"; do
  jq -r '.[]._source.layers | .usb."usb.src", .usb."usb.dst", ."usb.capdata", ""' <"$f"
done | while read src; read dst; read pl;read x; do
  [ "$pl" != "null" ] || continue

  if [ "$drop_in_lines" = 1 -a "$dst" = "host" ]; then
    echo "# ignoring incoming data: $pl"
    continue
  else
    drop_in_lines=
  fi

  if [ "$src" = "host" ]; then
    echo
    case "$pl" in
      93:01:01:*) echo "#: nmeaSwitch";;
      93:05:04:*) echo "#: model";;
      93:0a:*) echo "#: identification";;
      93:0b:03:*) echo "#: count";;
      93:05:07:*)
        IFS=: read c1 c2 c3 s1 s2 c5 c6 p1 p2 p3 rest <<.e
$pl
.e
        echo "#: read (size=$(printf "%04x" 0x$s1$s2), pos=$(printf "%06x" 0x$p1$p2$p3))"
        ;;
      93:09:*)
        IFS=: read c1 c2 ns8 ns7 ns6 ns5 ns4 ns3 ns2 ns1 s5 s4 s3 s2 s1 ck <<.e
$pl
.e
        echo "#: set_time (ns=$(printf "%d" 0x$ns1$ns2$ns3$ns4$ns5$ns6$ns7$ns8))"
        ;;
      93:11:02:*) echo "#: delete/reboot"; drop_in_lines=1;;
    esac
    echo "> $pl"
  elif [ "$dst" = "host" ]; then
    echo "< $pl"
  fi
done


# ./replay-prepare.sh gt-120b-kvm-sesson-20250529.json >gt-120b-kvm-sesson-20250529.json.txt
# ./replay-prepare.sh gt-120b-kvm-sesson-20250603.json >gt-120b-kvm-sesson-20250603.json.txt
