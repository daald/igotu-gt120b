import math
import re

prog = re.compile(r'([0-9]+(?:\.[0-9]+)?)')


def compare_numbers(num_str1: str, num_str2: str) -> bool:
    """
    Compares two number strings to see if they are within 1% of each other.
    """
    try:
        f1 = float(num_str1)
        f2 = float(num_str2)
        return math.isclose(f1, f2, rel_tol=0.01)
    except (ValueError, TypeError):
        return False


def fuzz_match(line1: str, line2: str) -> bool:
    """
    Performs a fuzzy match on two strings.

    It checks if the non-numeric parts are identical and the numeric parts
    are approximately equal (within a 1% tolerance).
    """

    result1 = list(prog.finditer(line1))
    result2 = list(prog.finditer(line2))
    n = len(result1)
    if n == 0: return False
    # don't continue if we don't have the same count of numbers
    if n != len(result2): return False

    # assert same prefix
    if result1[0].start() != result2[0].start(): return False
    if line1[0:result1[0].start()] != line2[0:result2[0].start()]: return False
    # assert same suffix
    if line1[result1[n - 1].end():] != line2[result2[n - 1].end():]: return False
    # assert all nun-numeric parts between match
    for i in range(0, n - 1):
        if (line1[result1[i].end():result1[i + 1].start()] !=
                line2[result2[i].end():result2[i + 1].start()]): return False

    for i in range(0, n):
        if not compare_numbers(result1[i].group(), result2[i].group()): return False

    return True


def fuzz_compare(a, b):
    return a == b or fuzz_match(a, b)


# --- Tests ---
def check_no_match(a, b):
    assert not fuzz_compare(a, b), f"Should not match but did: {a} vs {b}"


def check_match(a, b):
    assert fuzz_compare(a, b), f"Should match but didn't: {a} vs {b}"


def test_fuzz_compare():
    # Test assertions
    check_match("<a>1.234</a>", "<a>1.234</a>")
    check_match("<a>1.234</a>", "<a>1.23399999</a>")
    check_no_match("<a>1.234</a>", "<a>1.274</a>")
    check_no_match("<a>1.23399999</a>", "<a>1.274</a>")
    check_no_match("<a>1.274</a>", "<b>1.274</a>")
    check_match("<a>0</a>", "<a>0.0</a>")
    check_no_match(' <trkpt lat="36.888621" laan="5.485955">', ' <trkpt lat="36.888622" lon="5.485955">')
    check_match(' <trkpt lat="36.888621" lon="5.485955">', ' <trkpt lat="36.888622" lon="5.485955">')
    check_no_match(' <trkpt lat="36.888621" lon="7.485955">', ' <trkpt lat="36.888622" lon="5.485955">')
    check_no_match("a11:22b", "a10:22b")
    check_match("a11:22b", "a11:22b")

    print("All tests passed! ✅")


if __name__ == "__main__":
    test_fuzz_compare()
