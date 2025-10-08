#!/bin/sh

set -ex

if [ -z "$1" ]; then
  for f in *.json.txt; do
    "$0" "$f" >"$f.summarized"
  done
  exit 0
fi

read_rest() {
  local fullline=""
  while read line3; do
    [ "${line3#<}" != "$line3" ] || break
    fullline="$fullline:${line3#< }"
  done
  fullline="${fullline#:93:??:??:}"
  fullline="${fullline%:??}"
  if echo "${fullline}:" | grep -qP '^(ff:)+$'; then
    echo "EMPTY"
  else
    echo "(data)"
  fi
}

cat "$1" | while read line; do
  case "$line" in
    "#: read"*)
      part1="$line"
      read line2
      printf "%-40s %-40s " "$line" "$line2"
      read_rest
      ;;

    "#: delete/"*)
      part1="$line"
      read line2
      printf "%-40s %-40s %s\n" "$line" "$line2"
      ;;

    "#: "*)
      part1="$line"
      read line2
      read line3
      printf "%-40s %-40s %s\n" "$line" "$line2" "$line3"
  esac
done
