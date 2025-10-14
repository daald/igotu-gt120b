import sys

import difflib

"""
Custom implementation of diff -u.
It is made to ignore some custom differences in gpx files, if you compare the
windows original software output with our output. The usual (and allowed)
differences are:
- file ending: the last line may or may not contain a last newline
- numbers: the biggest thing: the original sw uses a weak float implementation
  thus, we use some fuzzy compare (max 1% difference in value)
- comment line: in tests, we include a comment line which we ignore for the
  diff
"""


def prepare_file(filename):
    lines = open(filename).readlines()
    lines = [s for s in lines if not s.startswith(header_to_remove)]
    if not lines[-1].endswith("\n"): lines[-1] += "\n"
    lines[-1] = lines[-1].rstrip()
    return lines


if __name__ == '__main__':
    header_to_remove = "<!-- generated using test of rust implementation -->"

    a_name = sys.argv[1]
    file_1 = prepare_file(a_name)
    b_name = sys.argv[2]
    file_2 = prepare_file(b_name)

    delta = difflib.unified_diff(file_1, file_2, a_name, b_name)
    sys.stdout.writelines(delta)
