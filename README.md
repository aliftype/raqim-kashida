# raqim-kashida

A library for finding _kashida_ (_tatweel_) insertion points and priorities,
driven by a small pattern language.

Given a word and a compiled pattern set, the crate returns the possible
_kashida_ insertion points and their priorities:

```rust
use kashida::{builtin_pattern_set, find_kashida_points};

let set = builtin_pattern_set("arabic-simple").unwrap();
let (cleaned, points) = find_kashida_points("بيت", set, true);
for point in points {
    // A kashida can be inserted after the grapheme cluster `point.index`.
    println!("{} @ {}", point.priority, point.index);
}
```

The _kashida_ insertion points are detected based on text analysis and
_kashida_ rules. It does not take fonts or shaping into account. The actual
justification is also out of scope. It is up to the caller to use the _kashida_
points to justify the text.

## API

The API is documented at [docs.rs](https://docs.rs/raqim-kashida).

## Pattern sets

Pattern sets describe where _kashida_ can be inserted and the priority of each
insertion point.

They are compiled from a textual representation loosely inspired by Knuth-Liang
hyphenation patterns.

The [built-in sets](data/) are written in the same language. Callers can
compile their own with `compile_pattern_text()`.

### Lines

A set textual representation is a sequence of lines. Blank lines and everything
after a `#` are ignored:

```
# Elongate after an initial or medial seen.
@Seen 6 *      # trailing comments work too
```

Each non-blank line is a **pattern**.

### Grammar

```
text      ::= line ("\n" line)*
line      ::= pattern? comment?
comment   ::= "#" anything

pattern   ::= guard? element+
guard     ::= "[" bound ("+" | "-" bound)? "]"
element   ::= token | weight | "."

token     ::= reference | set | "^" (set | reference) | letter | "*"
set       ::= "{" member+ "}"
member    ::= reference | letter
reference ::= ("@" | "=") name

weight    ::= digit ("\" digit)? | "!"

name      ::= (ALPHA | "_")+
bound     ::= digit+
letter    ::= a codepoint with a joining Joining_Type
```

Whitespace is never significant, except that it separates members inside
`{…}`. Beyond the grammar:

- A pattern needs at least one token. A `.` before the first token matches
  the run start, after the last token the run end, and nothing may follow a
  trailing `.`.
- Each gap between tokens holds at most one weight, and a gap outside a `.`
  boundary holds none.
- `name` is a canonical Unicode `Joining_Group` long name, or `Tatweel` for
  the tatweel itself.
- Guard bounds are at least 2, and a range’s low bound must not exceed its
  high one.
- A weight’s second digit must not exceed its first.

### Pattern lines

A weight sits at the **connection** before the token that follows it. A weight
after the last token applies to the connection after it. A pattern matches a run
of letters starting at some position when every token matches the corresponding
grapheme, and each weight then contributes to its connection.

#### Tokens

| Token          | Matches                                             |
| -------------- | --------------------------------------------------- |
| `@Name`        | a Unicode `Joining_Group` by long name              |
| `=Name`        | that `Joining_Group` alone, with no folding         |
| `@Tatweel`     | the tatweel U+0640 itself                           |
| `{@G1 @G2 …}`  | any one of the listed members                       |
| `^{@G1 @G2 …}` | any joining letter matching **none** of the members |
| letter         | that exact letter only                              |
| `*`            | any joining letter                                  |
| `.`            | a joined-run boundary                               |

- **`@Name`** names a Unicode `Joining_Group` by its exact long name (e.g.
  `@Beh`, `@Teh_Marbuta`, `@Farsi_Yeh`). It folds **positionally** through the
  rasm classes. For example: `@Beh` also matches the whole beh family as well
  as noon and yeh in initial/medial positions. A group that does not fold just
  matches itself alone.
- **`=Name`** matches that `Joining_Group` alone, in any position, so `=Beh` is
  beh joining group and nothing else.
- **`@Tatweel`** is the one non-group name: the tatweel U+0640 itself, as an
  exact literal.
- **Letters** (`ب ت س ك …`) match only themselves. Use `@Name` or a group-set
  to match a whole group.
- **`.`** matches the boundary of the joined run (i.e its start when leading,
  its end when trailing) not the word’s edge: e.g. `@Waw .` is a heh that ends
  its joined run (a final or isolated waw) regardless if its position in the
  word.

Anything else in a pattern line (punctuation, a Latin letter, an Arabic-Indic
digit, any character that could never match a joining letter) is a compile
error, not a token.

#### Length guards

A `[…]` prefix restricts a pattern to joined runs of a given **letter count**
(marks excluded):

| Guard   | Run length  |
| ------- | ----------- |
| `[4]`   | exactly 4   |
| `[4+]`  | 4 or more   |
| `[2-3]` | 2 through 3 |

The length is the joined run length, not the word length. In “المبتعث” the “ا”
is its own joined run and “لمبتعث” is another joined run.

The bounds must be whole numbers, at least 2 (no shorter run has a connection),
and a range must not be empty. With no guard a pattern applies at any length.

#### Priority

A digit `0`–`9` between two tokens is the **priority** of a kashida at that
connection, higher meaning more preferable. At each connection the **highest**
priority across all matching patterns wins and is the one reported.

An **absent** digit is not a candidate at all. An explicit `0` is the weakest
candidate (lowest priority). One gap holds at most one weight;
two digits, or a digit over a `!`, in the same gap is a compile error, as is a
weight in the gap between a token and a `.` (no connection exists at a run’s
edge).

```
@Seen 6 *       # after an initial/medial seen, priority 6
ب 0 ت           # a beh→teh connection, weakest possible candidate
```

##### Length-dependent priority

A priority written as two digits (`9\6`) drops as the run grows. For example:

```
[4+] @Beh 9\6 @Ain      # a kashida between a beh and an ain
```

The `[4+]` guard matches joined runs of four letters or more, so four is this
rule’s **floor length**, where the priority is the first digit; each letter
beyond the floor lowers the priority by one until it reaches the second digit:

| Joined run | Length | Priority |
| ---------- | ------ | -------- |
| بعثة       | 4      | 9        |
| مبتعث      | 5      | 8        |
| لمبتعث     | 6      | 7        |
| لمبتعثة    | 7      | 6        |

So the same rule marks the connection as excellent in a short run and merely good
in a long one.

Without a guard the floor is 2, the shortest run with a connection. The second
digit must not exceed the first, and a plain digit is the constant case where
the priority is same at every length.

#### Suppression: `!`

A `!` where a digit would go **blocks** that point unconditionally. So no
candidate, regardless of any priority another pattern assigns there.
Suppression is top precedence.

```
@Seen 6 *           # seen normally elongates,
@Seen ! @Yeh .      # but not directly before a final yeh
@Lam ! *            # lam never hosts a kashida
```

### Inline group-sets

`{@Group1 @Group2 …}` matches against **any** of its members, written inline at
the point of use. A member is written exactly like a standalone token (an
`@Name` or `=Name` reference, `@Tatweel`, or a literal letter). A leading `^`
negates the set: `^{…}` matches any joining letter in **none** of the listed
groups (and `^@Name` or `^=Name` is the single-group complement).

```
* 9 {@Heh @Dal} .               # a final heh-family letter (Heh folds) or dal
@Seen 8 ^{@Yeh @Farsi_Yeh} .    # any final but a yeh
```

### Point placement

A kashida at the connection between graphemes `i` and `i+1` is inserted before
grapheme `i+1`’s cluster. A point is a candidate only where there is a real
connection: never after a right-joining letter such as `د ر و`, and never
across one a ZWNJ suppresses. Suppressing kashida inside lam-alef ligature, for
instance, is up to the pattern itself.

## License

Licensed under the [MIT license](LICENSE).
