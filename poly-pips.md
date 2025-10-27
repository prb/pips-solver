# Polypips Generator / Solver
This document describes a new game called "Polypips" that is a portmanteau of "polyomino" and "pip".  The idea is to extend the concepts of the Pips game to [polyominoes](https://en.wikipedia.org/wiki/Polyomino).  ([Polyominoes 101](https://polyominoes.co.uk/polyominoes/101/index.html#3) also provides a good introduction and inspired the generation of the 8x8-N puzzle generator, for example.)

This document builds on the approach from the Pips solver described in [pips-solution-strategy.md](pips-solution-strategy.md).

## Goals / Executables

### General Puzzle Generation
The `generate_polypips` commandline tool generates a solvable Polypips puzzle in the form of a board, constraints, and pieces — in a variety of default shapes or in a custom shape supplied by the user.  The generator output should use the brief notation below for the pieces.  The arguments and flags for the executable will be described in detail below.

### Piece Visualization
The `draw_polypips` commandline tool displays the ASCII art representation of the polyominoes passed on the commandline and accepts a `--compact` flag for compact output.  See section on notation below.

Example invocations:

```
% draw_polypips 5Z-:12345
    ┌───────┐
    │ 2   1 │
┌───┘   ┌───┘
│ 4   3 │
│   ┌───┘
│ 5 │
└───┘
% draw_polypips --compact 5Z-:12345
 21
43
5
```

### Puzzle Solving
- `solve_polypips`: Solve a Polypips puzzle based on an input in the form outputted by the generator.

## Polyomino Notation

Note that for our purposes, we imagine the polyominoes being like physical dominoes with the pieces being marked on one face with pips and blank on the other.  Placing the pieces allows for 0-90-180-270 degree rotations but not reflections, and with this in mind, we distinguish chirality (i.e., left- and right-handed) for polyominoes.

### Input
We use the following brief input notation for polyominoes with pips as labels on the squares.  The notation is of the form

```
[number of squares][shape][chirality, if relevant]:[pips][pips]...
```

#### Binding of Pips to Polyomino Cells and Possible Board Placements

The pips in the notation are bound to polyomino cells in the order specified below, and we use a layout-sensitive notation to specify the shapes with a `|` denoting the left edge and a `.` for an unoccupied square.  For example, the definition of the `5W` polyomino is:

```
5W:mnopq  => |m..
             |no.
             |.pq
```

In terms of assignments of pips to coordinat offsets in our left-right/up-down system, this would be:

```
{((0,0),m), ((0,1),n), ((1,1),o), ((1,2),p), ((2,2),q)}
```

For a more concrete example, the `5W:12345` polyomino would be:

```
5W:mnopq  => |1..
             |23.
             |.45
```

And then the four possible board placements of the `5W:12345` would be as follows, where we use the `:d` suffix for `d` in `{0,90,180,270}` to denote degrees of notation and omit `:0` for brevity at times:

```
# 0 degrees
5W:12345:0  => |1..
               |23.
               |.45

# 90 degrees
5W:12345:90  => |.45
                |.3.
                |12.

# 180 degrees
5W:12345:180  => |5..
                 |43.
                 |.21

# 270 degrees
5W:12345:270  => |.21
                 |.3.
                 |54.
```

One way to make the rotation operation systematic would be to treat the polyomino as a matrix of values with `-1` (an integer not in the set of possible pips)representing an empty square, e.g.:

```
5W:12345:0  => [  1 -1 -1 ]
               [  2  3 -1 ]
               [ -1  4  5 ]
```

And then the 90 degree rotation is just the 90-degree rotation of the matrix:

```
5W:12345:0  => [ -1  4  5 ]
               [ -1  3 -1 ]
               [  1  2 -1 ]
```

Note that not all shapes have square matrices, e.g., `4L+:3456`:

```
4L+:3456:0  => [  3 -1 -1 ]
               [  4  5  6 ]
```

In which case, we would have:

```
4L+:3456:90  => [ -1  6 ]
                [ -1  5 ]
                [  3  4 ]
```

#### Monomino Notation
The notation `1o:m` represents a 1x1 polyomino (AKA a monomino) with `m` pips.  There is only one distinct shape.

```
1o:m = |m
```

#### Domino Notation
The notation `2s:mn` represents a 2x1 polyomino where `m` and `n` are the pips and `s` is always `I`.  There is only one disinct shape.

```
2I:mn => |mn
```

#### Triomino Notation
The notation `3s:mno` represents a 3x1 polyomino where `m`, `n`, and `o` are the pips and `s` is one of `IL`.  As above, each of the shape letters represents a layout of the squares of the polyomino.  There are two distinct shapes

```
3I:mno => |mno

3L:nmo => |m.
          |no
```

#### Tetromino Notation
The notation `4s:mnop` represents a 4x1 polyomino where `m`, `n`, `o`, and `p` are the pips and `s` is one of `ILOST`.  As above, each of the shape letters represents a layout of the squares of the polyomino.  There are seven distinct shapes considering chirality or five independent of chirality.

```
4I:mnop  => |mnop

4L+:mnop => |m..
            |nop

4L-:mnop => |..m
            |pon

4O:mnop  => |mn
            |op

4T:mnop  => |mno
            |.p.

4S+:mnop => |mn.
            |.op

4S-:mnop => |.nm
            |po.
```

The `+`/`-` variants are mindfully constructed so that the numbered layouts are reflections across the vertical axis.  The reader will likely recognize these as the [Tetris](https://en.wikipedia.org/wiki/Tetris) shapes.

The positive/negative chirality of `LS` tetrominoes needs separate treatment for us, as the assignment of pips to locations can make the pieces different.

#### Pentomino Notation
The notation `5:mnopq` represents a 5x1 polyomino where `m`, `n`, `o`, `p`, and `q` are the pips and `s` is one of `FILPNTUVWYZ`.  (We follow the original [Golomb](https://en.wikipedia.org/wiki/Solomon_W._Golomb) mneunomic here; see the Wikipedia article on [pentominoes](https://en.wikipedia.org/wiki/Pentomino) for more history and discussion.)  As above, each of the shape letters represents a layout of the squares of the polyomino.  There are 18 distinct shapes considering chirality or 12 independent of chirality.

The positive/negative chirality of `FLNPYZ` pentominoes needs separate treatment for us because we only allow rotations of pieces when considering placement.

```
5F+:mnopq => |.mn
             |op.
             |.q.

5F-:mnopq => |nm.
             |.po
             |.q.

5I:mnopq  => |mnopq

5L+:mnopq => |m...
             |nopq

5L-:mnopq => |...m
             |qpon

5N+:mnopq => |mn..
             |.opq

5N-:mnopq => |..nm
             |qpo.

5P+:mnopq => |mn
             |op
             |q.

5P-:mnopq => |nm
             |po
             |.q

5T:mnopq  => |mno
             |.p.
             |.q.

5U:mnopq  => |m.n
             |opq

5V:mnopq  => |mno
             |p..
             |q..

5W:mnopq  => |m..
             |no.
             |.pq

5X:mnopq  => |.m.
             |nop
             |.q.

5Y+:mnopq => |mnop
             |.q..

5Y-:mnopq => |ponm
             |..q.

5Z+:mnopq => |mn.
             |.o.
             |.pq

5Z-:mnopq => |.nm
             |.o.
             |qp.
```

As with the tetrominoes, the `+`/`-` variants are mindfully constructed so that the numbered layouts are reflections across the vertical axis.

### ASCII Art Output
The notation above is adequate for outputting a polypip, but it is more useful for machine input/output than for human consumption.  For pretty-printing, we use an ASCII art style similar to the one used for the Pips solver.

For example, in the default orientation, the `5F+:12345` piece in isolation would be represented as:

```
    ┌───────┐
    │ 1   2 │
┌───┘   ┌───┘
│ 3   4 │
└───┐   │
    │ 5 │
    └───┘
```

## Polypips Game Generation

### Game Generation Inputs
The input for game generation is as follows:

1. A board shape, represented internally as a collection of pairs of non-negative integers, as in the Pips solver, with `x` coordinates numbered starting at `0` and increasing to the right, and `y` coordinates numbered starting at `0` and increasing downwards.
2. A description of the allowed piece polyominoes and rules for placement.
3. A description of the allowed constraint polyominoes and rules for placement.



These are discussed in more detail below.

#### Board Shape

We support board shape input in the same style as the Pips solver with `#` marks to show which points are part of the board.  For example, an `8x8` board with a `2x2` square hold in the middle would be:

```
########
########
########
###  ###
###  ###
########
########
########
```

### Polyomino Selection Rules

For pieces or constraints, we specify rules in the following formats:

```
[rule_type]:none|any|[number*|numbershape],[number*|numbershape],...
```

where `rule_type` is one of `pieces` or `constraints` and `number*` is one `{1*,2*,3*,4*,5*}` (intending to be any polyomino with that number of cells) and `numbershape` is one of the polyomino shapes from our short notation above.  If chirality is omitted, it is assumed to be unconstrained.  The `rule_type` of `none` is only supported for constraints.

For pieces, we support a special piece rule `12x5` that restricts the piece placements to exactly one of each of the twelve pentomino shapes up to chirality.

For constraints, we support an additional parameter of `constraint-coverage` that specifies the proportion of cells desired to be covered by the constraint polyominoes as a decimal value between 0 and 1.

For both constraints, we support a `constraint-selection` parameter that specifies the relative weight of polyominoes to be selected at random.  The two possible values are `uniform-size` and `uniform-all` with `uniform-all` being the default.  The behavior of `uniform-size` is to first select a constraint size and then select a constraint of that size from the available rules.  (For a specifier like `constraints: 3*,4*`, only `3` and `4` sizes are possible.)  The behavior of `uniform-all` is to select a constraint shape at random from all of the available constraints.  For example, with `constraints: any`, the probability of selecting `4O` is `1/29`, while with `constraint-selection: uniform-size`, the probability of selecting `4O` is `1/5*1/7=1/35`.

Some examples:

```
# Similar to Pips; constrain half the board (keep it chill-ish, more likely to select larger constraints)
pieces: 2*
constraints: any
constraint-coverage: 0.5

# Only 3L tri-ominoes with square (4O) constraints covering the whole board
pieces: 3L
constraints: 4O
constraint-coverage: 1.0

# Any tetronomino plus the X pentomino, domino constraints only; constrain most of the board
pieces: 4*,X
constraints: 2*
constraint-coverage: 0.8

# Just the basic 12 pentominoes with no constraints
pieces: 12x5
constraints: none
```

### Game Generation Algorithm
To generate a game, we proceed in phases:

1. Tile the board with abstract polyominoes (i.e., without pips assigned) representing the pieces and subject to any rules provided on the pieces allowed. As a tiling, this must exhaust all of the spaces on the board, so the work can proceed from the upper left corner of the board.  The ordering of the tiles at each step of the backtracking should be randomized (but deterministic for that location) to ensure an interesting tiling.
2. Perform a second pass over the board, placing non-overlapping abstract polyominoes that will be constraints, subject to rules provided on the constraint shapes allowed.  The constraints may not fully exhaust the spaces on the board, depending on the `constraint-coverage` parameter.  Unlike pieces, where placement can proceed in an orderly manner from the upper left with the halting condition being the exhaustion of the board, the points explored for constraint placement should be randomized around the board with the halting condition being meeting the `constraint-coverage` expectation.  (This should generate a more pleasing layout.)
3. Randomly assign a constraint expression to each constraint polyomino, ensuring that the expression is valid for the shape.  (E.g., a `3L` shape can't have a `>19` constraint.)
4. For each constraint polyomino, randomly assign pips to the underlying points in the board in a manner that satisfies the constraint.
5. For any points on the board not covered by a constraint polyomino, randomly assign pips.
6. Assign the pips from the board points to the piece polyominoes in the tiling.

This guarantees a solvable game by construction.  It may not be possible to tile the board with the given pieces or to meet the constraint coverage ratio, in which case an informative error message should be returned.

### Game Output

The game output should have the following structure:

```
board:
[board from the input]

pieces:
[list of pieces, comma-separated and trimmed to width]

constraints:
[list of constraints, one per line]

game:
[ASCII art of the board and constraints, as in the Pips solver]

[ASCII art of the pieces]

solution:
[ASCII art] for the solved board from the generator
```
