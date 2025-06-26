#!/bin/sh

set -e

# * sudo modprobe usbmon && sleep 1 && sudo chgrp wireshark /dev/usbmon3 && sudo chmod g+rw /dev/usbmon3
# * record using wireshark
# * usb.capdata[0] == 0x93 and ((usb.urb_type == 'S' and usb.transfer_type == 0x03 and usb.endpoint_address == 0x01) or (usb.urb_type == 'C' and usb.transfer_type == 0x03 and usb.endpoint_address == 0x81))
# * usb.addr == "3.7.1" or usb.addr == "3.8.1"

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
        IFS=: read c1 c2 us8 us7 us6 us5 us4 us3 us2 us1 s5 s4 s3 s2 s1 ck <<.e
$pl
.e
        echo "#: set_time (us=$(printf "%d" 0x$us1$us2$us3$us4$us5$us6$us7$us8))"
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
