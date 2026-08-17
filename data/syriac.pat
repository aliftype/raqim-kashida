# Built-in kashida pattern set for Syriac. Based on the guidelines in:
# [Spec] "Syriac Script Alignment - Justify.pdf", written by an expert in
# the language:
# https://bugs.documentfoundation.org/attachment.cgi?id=182206
# from:
# https://bugs.documentfoundation.org/show_bug.cgi?id=140767
#
# Rules numbered as [Spec] orders them, by precedence (rule 1 strongest), but
# written weakest first, since a later rule overrides an earlier one.
# It gives no per-letter priorities, so rules 1 and 2 are ladders. Each
# rung's length guard is what stops it at the midpoint. The order is exact for
# runs of 2 to 10 letters, beyond that the deepest points share the lowest
# priority. [Spec] measures over the word, we measure over each joined run.

# Whatever the ladders do not reach is still a candidate, just the weakest.
* 0 *

# Rule 1: between the before last and last character of a word by default,
# then between the before last letter and the letter before it, and so on up
# to the midpoint of the word.
* 8 * .
[4:] * 7 * * .
[6:] * 6 * * * .
[8:] * 5 * * * * .
[10:] * 4 * * * * * .

# Rule 2: if that is not possible, between the first letter and the letter
# after it, then between the second letter and the one after it, and so on.
[3:] . * 3 *
[5:] . * * 2 *
[7:] . * * * 1 *


# Rule 3: inter-word justification. Not relevant here.

# Rule 4: an automatic kashida appears after a manually user inserted
# kashida character, and not before it.
@Tatweel 9

# Rule 5: inserted Kashida characters should be distributed equally among all
# the words in a line. Not relevant here.

# Rule 6: no kashida between the letter sequence lomadh, olaph.
@Lamadh ! @Alaph

# Rule 7: spacing. Not relevant here.

# Rule 8 (no kashida after olaph, dolath, he, waw, zayn, sodhe, rish, taw or
# dotless dolath rish) and rule 9 (only letters take one between them).
# Those are all right-joining letters, so no connection follows one anyway,
# and marks belong to the letter they sit on.
