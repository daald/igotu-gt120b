#!/bin/bash

set -ex
set -o pipefail

test_with_directory() {
  local d="$1"
  local jsonfile="$(ls -d "$d/"*.json || true)"
  [ -f "$jsonfile" ] || { echo "ERROR: no .json file"; return; }
  local jsontxtfile="$jsonfile.txt"
  if ! [ -f "$jsontxtfile" ]; then
    echo "ad-hoc generating .json.txt file"
    helpers/bin/prepare-replay-txt.sh "$jsonfile" >"$jsontxtfile"
  fi
  local summaryfile="$jsonfile.txt.summary"
  if ! [ -f "$summaryfile" ]; then
    echo "ad-hoc generating .json.txt.summary file"
    helpers/bin/summarize-replay-txt.sh "$jsontxtfile" >"$summaryfile"
  fi
  [ -f "$jsontxtfile" ] || { echo "ERROR: no .json.txt file"; return; }
  if ! grep -q "#: delete/reboot" "$jsontxtfile"; then
    echo "ERROR: no standard case / no reboot in .txt"
    return
  fi
  if "$0" "$jsontxtfile"; then
    echo "OK"
  else
    echo "EXIT $?"
  fi
}

if [ ! -f "$1" ]; then
  [ -n "$logdir" ] || logdir=.
  [ ! -d ../../usbmon-log/ ] || logdir=../../usbmon-log
  [ ! -d ../datalogs/ ] || logdir=../datalogs
  case "$1" in
    all)
      testsummaryfile=/tmp/.$$.testlog
      for d in $logdir/gt-120b/*/; do
        [ -d "$d" ] || continue
        [ ! -f "$d/no-standard-case.stamp" ] || continue
        echo
        echo "## TESTING $d"
        test_with_directory "$d" >"$d/last-test.log" 2>&1
        lastline="$(tail -n30 "$d/last-test.log" | grep -F "Next cmd doesn't match" | tail -n1 | cut -c1-40)" ||:
        [ -n "$lastline" ] || lastline="$(tail -n5 "$d/last-test.log" | grep -i '^error' | tail -n1 | cut -c1-40)" ||:
        [ -n "$lastline" ] || lastline="$(tail -n5 "$d/last-test.log" | grep -vxF '+ return' | tail -n1)"
        printf "%-40s %s\n" "$lastline" "$d" >>"$testsummaryfile"
        printf "%-40s %s\n" "$lastline" "$d"
      done
      echo
      echo "Complete summary:"
      cat "$testsummaryfile"
      exit 0
      ;;
    d|*)
      exec "$0" $logdir/gt-120b/gt-120b-kvm-session-2025-07-31_anonymenoughforpub/gt-120b-kvm-sesson-20250731_rawdatacount3+3.filtered.pcapng.json.txt
      ;;
  esac
  exit 1
fi

panic() {
  echo "ERROR: $*" >&2
  exit 1
}

replayfile="$1"
[ -f "$replayfile" ]
basename="$(dirname "$replayfile")"
[ -d "$basename" ]

# normalizes numbers (round to 3 significant digits) and times (same format)
process_gpx_file() {
set +x
python3 -c "
import fileinput
import re
from math import log10, floor

from datetime import datetime # python>=3.11  - datetime.fromisoformat
import dateutil.parser  # python<3.11  - dateutil.parser.isoparse (needs sudo apt-get install python3-dateutil)


def round_to_sign_digits(x, decimal_places=3):
    '''round to n significant digits'''
    return round(x, -int(floor(log10(abs(x)))) - 1 + decimal_places)


def round_floats_in_stream(decimal_places):
    '''round all numbers to a number of significant digits. eg (with 3) 1234.567 -> 1230, 1.234567 -> 1.23, 0.01234567 -> 0.0123'''
    # Regular expression to find float numbers
    float_pattern = r'-?\d+\.\d+'
    time_pattern = r'20\d\d-?.*[\dZ]'

    def time_match(match):
        s = match.group()
        t = dateutil.parser.isoparse(s)  # only python >= 3.11 supports full iso format
        return t.isoformat()

    # Function to round the float numbers
    def round_match(match):
        number = float(match.group())
        s = str(round_to_sign_digits(number, decimal_places))
        if s.endswith('.0'): s = s[:-2]
        return s

    for line in fileinput.input():
        if '<!-- generated using' in line:
            continue
        elif '<time' in line:
            # Replace float numbers with their rounded versions
            outline = re.sub(time_pattern, time_match, line)
        else:
            # Replace float numbers with their rounded versions
            outline = re.sub(float_pattern, round_match, line)

        print(outline, end='');


# Usage
round_floats_in_stream(decimal_places=2)
"
set -x
}

expect_filelist="$(mktemp)"
sim_filelist="$(mktemp)"
file_actual="$(mktemp --suffix=-actual)"
file_expected="$(mktemp --suffix=-expected)"

ls "$basename"/gpx/*.gpx "$basename"/*.gpx 2>/dev/null >"$expect_filelist" || true
[ -s "$expect_filelist" ] || panic "No gpx found for comparing"
expect_filedir="$(head -n1 "$expect_filelist")"
expect_filedir="${expect_filedir%/*}"

#numgpx="$(ls "$basename"/gpx/*.gpx "$basename"/*.gpx | wc -l)"
#[ "$numgpx" -ge 1 ]

out_gpx=./  #"$(mktemp -d)"
[ -d gpx-bak/ ] || mkdir gpx-bak/
mv -v "$out_gpx"/testout-*.gpx gpx-bak/ || true
./simtest.sh "$replayfile"
ls "$out_gpx"/testout-*.gpx >"$sim_filelist"
[ -s "$sim_filelist" ] || panic "No output files found"

set +x
helpers/bin/gpx_summary.py *.gpx
echo
helpers/bin/gpx_summary.py "$expect_filedir/"*.gpx | sort
set -x

ex="$(wc -l <"$expect_filelist")"
ac="$(wc -l <"$sim_filelist")"
[ $ex -eq $ac ] || panic "Unexpected number of output files. expected: $ex, actual: $ac"

exec 4<"$expect_filelist"
exec 5<"$sim_filelist"
n=0
while read -eu 4 expect_file && read -eu 5 sim_file; do
  n=$((n+1))
  echo "COMPARING EXPECT $expect_file vs SIM $sim_file"

  #ex="$(grep -v "<!-- generated using" "$expect_file" | wc -l)"
  #ac="$(grep -v "<!-- generated using" "$sim_file"    | wc -l)"
  #[ $ex -eq $ac ] || panic "Different number of output lines in file $sim_file. expected: $ex, actual: $ac"

  ex="$(grep -c "<trkpt" "$expect_file")"
  ac="$(grep -c "<trkpt" "$sim_file")"
  [ $ex -eq $ac ] || panic "Different number of trackpoints in file $sim_file. expected: $ex, actual: $ac"

  python3 helpers/python/udiff.py "$expect_file" "$sim_file" || panic "Different output files $sim_file vs $expect_file"

  #diff -wu "$expect_file" "$sim_file"
  #meld "$expect_file" "$sim_file"

  #process_gpx_file <"$expect_file" | grep -v -e '<gpxtpx:' >"$file_expected"
  #process_gpx_file <"$sim_file"    | grep -v -e '<gpxtpx:' >"$file_actual"

  #diff -wu "$file_expected" "$file_actual" || panic "Different output files $sim_file vs $expect_file"
done

rm -f "$expect_filelist" "$sim_filelist" "$file_actual" "$file_expected"

echo "All tests passed. Checked $n output files"
exit 0
