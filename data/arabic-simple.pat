# Built-in kashida pattern set for simple Arabic fonts. Based loosely on
# Microsoft rules, as documented in:
# https://web.archive.org/web/20130308140133/microsoft.com/middleeast/msdn/JustifyingText-CSS.aspx
#
# Rules ordered by priority (rule 1 strongest)

# No kashida inside lam-alef ligature.
@Lam ! @Alef

# Rule 1: a kashida the user already typed is itself the strongest point.
@Tatweel 9

# Rule 2: after an initial or medial seen or sad.
{@Seen @Sad} 8 *

# Rule 3: before a final heh or dal.
7 {@Heh @Dal} .

# Rule 4: before a final alef, tah, lam, kaf, or gaf.
6 {@Alef @Tah @Lam @Kaf @Gaf} .

# Rule 5: before a medial beh followed by a final yeh, reh, or yeh.
5 @Beh {@Reh @Yeh @Yeh_Barree} .

# Rule 6: before a final waw, ain, qaf, or feh.
4 {@Waw @Ain @Qaf @Feh} .

# Rule 7: before any other final letter.
3 * .
