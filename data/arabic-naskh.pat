# Classical Arabic Naskh kashida pattern set, based largely on Benatia’s matrix:
# [B06] Benatia, Elyaakoubi & Lazrek (2006), TUGboat 27(2):137–146, figure 25
# and its legend in §3.4.
# with overriding rules from:
# [Afifi] Fawzi Salim Afifi, “Learning Arabic Calligraphy”, part 3.
#
# The matrix scores each (stretched letter, following letter) pair, encoded
# as a priority that drops with run length:
#   9\6  recommended  (Benatia "+")
#   6\3  neutral      (an unsigned number)
#   3\0  discouraged  (Benatia "−")
# and an empty cell compiles to no rule at all.
#
# The `[:4:]` guard means the priority is highest in a 4-letter run and drops
# by 1 for every letter away from that, either way. This is how we handle the
# [B06] §3.2 word-length rules: 4-letter words stretch best, 5 is disputed,
# and longer ones defer to the matrix. [B06] applies the matrix to runs of 4
# or more only. We apply it to runs of 2 and 3 as well, where the guard gives
# them a lower priority.


# [B06] §3.2: no kashida before a final yeh (handled by the Afifi rules at
# the end)

# [B06] §3.2 3-letter runs: the kashida follows the second letter
# when an ain, tah, seen, or hah leads (Ibn Al‘afif) or an alef or lam
# trails (Azzeftaoui).
[3] {@Ain @Tah @Seen @Hah} * 6 *
[3] * 6 * {@Alef @Lam}

# [B06] §3.4 figure 25, the matrix itself: within each
# block followers are grouped by priority (not the paper's column order).

# beh family
[:4:] @Beh 9\6 @Tah
[:4:] @Beh 6\3 {@Alef @Meem @Noon @Heh}
[:4:] @Beh 3\0 {@Hah @Dal @Reh @Lam}

# hah family
[:4:] @Hah 9\6 @Tah
[:4:] @Hah 3\0 {@Alef @Hah @Dal @Reh @Ain @Kaf @Lam @Meem @Noon @Heh @Waw}

# ain family
[:4:] @Ain 9\6 @Tah
[:4:] @Ain 3\0 {@Alef @Hah @Dal @Reh @Ain @Lam @Meem @Noon @Heh}

# seen, sad and tah families
[:4:] {@Seen @Sad @Tah} 6\3 {@Alef @Beh @Reh @Seen @Sad @Tah @Kaf @Lam @Noon}
[:4:] {@Seen @Sad @Tah} 3\0 {@Hah @Dal @Ain @Feh @Qaf @Meem @Heh @Waw}

# feh/qaf families
[:4:] @Feh 9\6 @Tah
[:4:] @Feh 6\3 @Alef
[:4:] @Feh 3\0 {@Hah @Dal @Reh @Ain @Lam @Meem @Waw}

# kaf and lam families (the Afifi rules suppress kashida after either, so its rows are
# recorded here and left commented out).
# [4:] @Kaf 6\3 {@Dal @Reh}
# [4:] @Kaf 3\0 {@Alef @Hah @Sad @Kaf @Lam @Meem @Noon @Heh @Waw}
# [4:] @Lam 3\0 {@Hah @Dal @Ain @Meem @Noon @Heh}

# meem
[:4:] @Meem 6\3 {@Tah @Dal @Reh}
[:4:] @Meem 3\0 {@Alef @Hah @Ain @Kaf @Lam @Meem @Noon @Heh @Waw}

# heh
[:4:] @Heh 6\3 @Beh
[:4:] @Heh 3\0 {@Seen @Reh @Dal @Heh}

# [B06] §3.2: a word ending in a heh pronoun is best stretched just before
# the heh. @Heh folds to the whole family (teh-marbuta is the heh bowl plus
# dots) in final/isolated.
* 9 @Heh .

# [Afifi]

# p. 5: no elongation occurs before the letters sad, ain, waw, heh, or a final feh.
# p. 10: no space may be stretched before a sad.
# p. 12: do not stretch a space before the letters feh, qaf, or waw.
# Hah below is my own, no source explicitly mentions it, but hah connects from top
# so can’t have a kashida.
! {@Hah @Sad @Ain @Waw}
#! @Heh * # Other sources allow kashida here, and it feel dubious.
! {@Feh @Qaf @Yeh @Yeh_Barree} .

# p. 5: no elongation occurs after the letter lam.
# p. 12: no elongation may be made after the kaf
# p. 14: no elongation may be made after lam.
# Muhammad Mu’nis in his al-Mizan al-ma’luf explicitly says lam is treated
# exactly like beh and gives examples of stretching initial and medial lam.
{@Kaf @Lam} !
