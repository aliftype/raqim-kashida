# Built-in kashida pattern set for Nastaliq.

use arabic-naskh

# From Afifi’s Silsilat taalim al-khatt al-arabi, part 9: al-khatt al-farisi.

# p. 94: A kashida after a lam, a kaf or an initial beh reads as a seen.
{@Kaf @Lam} !
. @Beh !

# p. 95: A kashida before these looks bad (does not join well: thick stroke of
# kashida where a thin stroke is needed).
! {@Ain @Feh @Qaf @Tah @Hah @Sad @Waw}
! @Heh *
! {@Yeh @Yeh_Barree} .
