This project is an initial exploration of Rust programming in the form of a solver for the NY Times game "Pips". Some useful links:

- [NY Times Pips Game](https://www.nytimes.com/puzzles/pips)
- [Pips Game Launch Post](https://www.nytimes.com/2025/08/31/briefing/pips-new-york-times-games.html)
- [Rust Programming Language](https://www.rust-lang.org/)

The game involves placing a list of dominos onto a pre-defined game board in a non-overlapping way that satisfies a set of color-coded constraints, covers the entire board, and places all of the dominos.  (The colors are purely decoration.)

# Project Goals

- Define an idiomatic, functional data model for the board, constraints, and pieces.
- Solve a game using a backtracking-style algorithm.
- Load a game from a simple textual representation in a file, solve it, and output the solution to standard output.

# Program Structure
The executable should be named `pips-solver`, and it should accept a file path as an argument.

The data model, game loader, and solver should all be in separate submodules.  For the data model, each struct should be defined in its own module.

Unit tests should be written for each of the components based on the examples in this specification document.

# Data Model
The fundamental idea for the data model is to treat the board as a two-dimensional grid with non-negative integer coordinates.

## Pips
A _pips_ is a non-negative integer in the range `[0..6]` that represents the number of "pips" or dots on half of a domino.

The `Pips` type should not allow the assignment of integers outside of the prescribed range.  Equality, hashing and ordering for `Pips` are inherited from the contained integer.

## Pieces / Dominoes
A _piece_ is an ordered pair of `Pips` that represents a domino.  For the purpose of comparison, two pieces are the same if they contain the same two `Pips`.  For simplicity and because there is no risk of confusion, we will occasionally write the pieces simply as a side by side integers, e.g., `12` represents the piece `(1,2)`.

To avoid confusion, we implement the Piece type by always storing the Pips in non-descending order.  For example, the piece `12` is stored as `(1,2)` and the piece `21` is stored as `(1,2)`; there is no ambiguity in storing `11` as `(1,1)`. This is intended to avoid difficulties with managing membership in collections.

A piece with the same pips for each of its positions is called a _doubleton_.  For example, `00`, `11`, `22`, `33`, `44`, `55`, and `66` are all doubletons.

We define a convenience function `removeOne` that accepts a `Vec<Piece>` and a piece and returns a `Result<Vec<Piece>,Err>` new list that removes a single occurrence of the piece from the list or an error if the piece is not found.  For example:
```
removeOne([Piece(1,2),Piece(1,2)], Piece(1,2)) = Ok([Piece(1,2)])
removeOne([Piece(3,4),Piece(5,6)], Piece(1,2)) = Err("(1,2) was not present in the list of pieces.")
```

## Points
A _point_ is an ordered pair of non-negative integers that represents a location on the grid.  Geometrically, the coordinate grid is imagined with increasing, zero-based x-coordinates moving left-to-right and increasing, zero-based y-coordinates moving top-to-bottom.  For example, `(0,0)` is the top-left corner of the grid, `(1,0)` is the point immediately to the right of `(0,0)`, and `(0,1)` is the point immediately below `(0,0)`.

## Board
A _board_ is a set of non-negative integer coordinate pairs.  Rather than defining or enforcing invariants of a board, we allow the solver to determine the validity of a board.

There is a special singleton board `EMPTY_BOARD` that represents a board with no points.  Two boards are equal if the contain the same set of points.

## Assignments
An _assignment_ is a pair of a pips and a point:

```
Assignment(Pips,Point)
```

## Constraints
A _constraint_ defines legal assignments as part of a game.  A constraint has a _constraint type_ that is one of `AllSame`, `AllDifferent`, `LessThan`, `Exactly`, or `MoreThan` plus additional data that depends on the type, as follows:

- For an `AllSame` constraint, there is an optional pips that represents the target pips that must be assigned to any point in the constraint.  We notate an `AllSame` constraint as `AllSame(Option<Pips>,HashSet<Point>)`.  The constraint only makes sense if the set of points is non-empty.
- For an `AllDifferent` constraint, there is a possibly empty set of pips that are not allowed to be assigned to any point in the constraint.  We notate an `AllDifferent` constraint as `AllDifferent(HashSet<Pips>,HashSet<Point>)`.  The set of pips must not contain all possible pips.  The constraint only makes sense if the set of points is non-empty.
- For a `LessThan`, `Exactly`, or `MoreThan` constraint, the intention is that the sum of all pips assigned to points in the constraint meets the comparison of the type name applied to a target non-negative integer value.  We notate the constraint as `<type>(NonNegativeInteger,HashSet<Point>)`.  Note that the sum of all pips might exceed the range of possible pips, so we use a non-negative integer rather than a `Pips` for this purpose.

As a notational convenience, we use the special singleton constraint `EMPTY_CONSTRAINT` to represent a constraint with no points.

For the purpose of comparison, two constraints are equal if they are of the same type and have the same components.  (Note that the invariants above ensure that there are no equivalent constraints of different types or different components, although this has no bearing on the behavior of the solver.)

A collection of constraints is _consistent_ if there is no point that appears in more than one constraint.  A constraint is _compatible_ with a board if all of the points in the constraint are also in the board.

## Constraint Invariants
There are the following additional invariants to consider.  These invariants should be enforced at construction time for a constraint.

### AllDifferent Invariants
For an `AllDifferent` constraint, the size of the set of pips to be avoided and the size of the set of points in the constraint must sum to no more than the size of the range of possible pips.  For example:

```
AllDifferent(
  {Pips(1),Pips(2),Pips(3),Pips(4),Pips(5)},{(0,0),(1,0),(2,0)}
)
```

would violate this invariant because `5+3>7`.

For an `AllDifferent` constraint with an empty set of values to be avoided, a set of points must have size at least `2`.  For example, the constraint `AllDifferent({}, {(0,0)})` would violate this invariant.

### AllSame Invariants
For an `AllSame` constraint, the set of values must be size at least `2`.

### LessThan Invariants
For a `LessThan` constraint, the target sum must not be less than the value of the least pips, i.e., `0`.  For example, the constraint `LessThan(0,_)` is not permitted.

For a `LessThan` constraint, the target sum must also be strictly less than the largest Pips (i.e., `6`) multiplied by the number of points in the constraint.  For example, the constraint `LessThan(18,_)` is not permitted for a constraint with `3` points.

### Exactly Invariants
For an `Exactly` constraint, the target sum must not be larger than the number of points in the constraint multiplied by the value of the largest possible pips, i.e., `6`.  For example, the constraint `Exactly(19,_)` is not permitted for a constraint with `3` points.  The constraint `Exactly(0,_)` would permitted for a constraint with `3` points.

### MoreThan Invariants
For an `MoreThan` constraint, the target sum must not be larger than the number of points in the constraint multiplied by the value of the largest possible pips, i.e., `6`.  For example, the constraint `MoreThan(18,_)` is not permitted for a constraint with `3` points.

### Constraint Reduction
We define a function `reduceA` that accepts a constraint `c` and an assignment `a` as arguments and returns an `Result<Constraint>`.

In the following definition, we elide the difference between a pips and the value it contains:

```
// Empty constraint reduces to itself.
reduceA(EMPTY_CONSTRAINT,_) = Ok(EMPTY_CONSTRAINT)

// No change if the point is outside the constraint
reduceA(c,a) = Ok(c), if a.point is not in c.points

// all_different
reduceA(AllDifferent(V,P),a) = Err("The pip {} is already used.", a.pips), if a.pips in V
reduceA(AllDifferent(V,P),a) = Ok(AllDifferent(V+{a.pips},P-{a.point})), if a.pips not in V and size(P) > 1
reduceA(AllDifferent(V,P),a) = Ok(EMPTY_CONSTRAINT), if a.pips not in V and size(P) == 1

// all_same
reduceA(AllSame(Some(v1),P),a) = Err("The pip {} is not the same as the expected pip {}.", a.pips, v1), if a.pips != v1
reduceA(AllSame(None,P),a) = Ok(AllSame(Some(a.pips),P-{a.point})), if size(P) > 2
reduceA(AllSame(None,P),_) = Ok(EMPTY_CONSTRAINT), if size(P) == 1
reduceA(AllSame(None,P),a) = Ok(Exactly(a.pips,P-{a.point})), if size(P) == 2
reduceA(AllSame(Some(v1),P),a) = Ok(AllSame(Some(a.pips),P-{a.point})), if a.pips == v1 and size(P) > 2
reduceA(AllSame(Some(v1),P),a) = Ok(Exactly(v1,P-{a.point})), if a.pips == v1 and size(P) == 2
reduceA(AllSame(Some(v1),P),a) = Ok(EMPTY_CONSTRAINT), if a.pips == v1 and size(P) == 1

// exactly
reduceA(Exactly(v1,P),a) = Err("The pips {} is not the same as the expected pip {}.", a.pips, v1), if P.points.size() == 1 and a.pips != v1
reduceA(Exactly(v1,P),a) = Ok(EMPTY_CONSTRAINT), if P.points.size() == 1 and a.pips == v1
reduceA(Exactly(v1,P),a) = Err("The pip {} exceeds the expected exact sum {}.", a.pips, v1), if a.pips > v1 and size(P) > 1
reduceA(Exactly(v1,P),a) = Err("The remaining sum {} is unachievable with {} points.", v1-a.pips, size(P-{a.point})), if a.pips <= v1 and size(P) > 1 and (v1-a.pips) > 6*size(P-{a.point})
reduceA(Exactly(v1,P),a) = Ok(Exactly(v1-a.pips,P-{a.point})), if a.pips <= v1 and size(P) > 1 and (v1-a.pips) <= 6*size(P-{a.point})

// less_than
reduceA(LessThan(v1,P),a) = Err("The pips {} is not less than the target sum {}.", a.pips, v1), if a.pips >= v1
reduceA(LessThan(v1,P),a) = Ok(EMPTY_CONSTRAINT), if a.pips < v1 and size(P) == 1
reduceA(LessThan(v1,P),a) = Ok(Exactly(0,P-{a.point})), if (v1-a.pips) == 1 and size(P) == 2
reduceA(LessThan(v1,P),a) = Ok(LessThan(v1-a.pips,P-{a.point})), if a.pips < v1 and size(P) >= 2

// more_than
reduceA(MoreThan(v1,P),a) = Err("The pips {} is less than the minimum required sum of {}.", a.pips, v1+1), if a.pips <= v1 and size(P) == 1
reduceA(MoreThan(v1,P),a) = Ok(EMPTY_CONSTRAINT), if a.pips > v1 and size(P) == 1
reduceA(MoreThan(v1,P),a) = Ok(Exactly(6,P-{a.point})), if (v1-a.pips) == 5 and size(P) == 2
reduceA(MoreThan(v1,P),a) = Ok(MoreThan(v1-a.pips,P-{a.point})), if a.pips > v1 and size(P) >= 2
```

## Game
A _game_ is a board, collection of pieces (which may include duplicates), and a potentially empty set of constraints.  A game is _valid_ if the number of coordinates in the board is double the number of pieces and the collection of constraints is consistent.

We define a unique singleton game `WON_GAME` as an empty board, empty set of pieces, and empty set of constraints.

## Directions

A _direction_ is one of the compass directions `North`, `East`, `South`, or `West`.

## Placements
A _placement_ is a piece together with a point and a direction:

```
Placement(Piece,Point,Direction)
```

A placement `p` has accessors `p.piece`, `p.point`, and `p.direction` for its components.

We define a function `assignments` that accepts a placement as argument and returns a list of assignments, as follows:

```
Placement(Piece(p1,p2), (q1,q2),North) = [Assignment(p1,(q1,q2+1)), Assignment(p2,(q1,q2))
Placement(Piece(p1,p2), (q1,q2),East)  = [Assignment(p1,(q1,q2)), Assignment(p2,(q1+1,q2))
Placement(Piece(p1,p2), (q1,q2),South) = [Assignment(p1,(q1,q2)), Assignment(p2,(q1,q2+1))
Placement(Piece(p1,p2), (q1,q2),West)  = [Assignment(p1,(q1+1,q2)), Assignment(p2,(q1,q2))
```

Note that there is no difference between the `North` and `South` or `East` and `West` directions for a doubleton.

We define a function `points` method that returns the set of points from the assignments of a placement.  For example:

```
Placement(Piece(0,1), (0,0), North).points() = {(0,0), (0,1)}
```

For a board `b`, a placement `p` is _legal_ if `p.points()` is a subset of `b.points()`.

### Placements and Constraints
For constraint `c` and a placement `p`, we define `reduceP(c,p)` as `reduceA` folded over `assignments(p)`.  We say that `p` _satisfies_ `c` if `reduce_P(c,p)` is `Ok`.

For example,

```
reduceP(
  AllSame(None,{(0,0), (0,1)}),
  Placement(Piece(0,1), (0,0), North)
)
  = reduceA(
      reduceA(
        AllSame(None,{(0,0), (0,1)}), Assignment(0,(0,1))
      ), Assignment(1,(0,0))
    )
  = reduceA(
      Some(Exactly(0,{(0,0)}))
      , Assignment(1,(0,0))
    )
= Err("The pip 1 is not the same as the expected pip 0.")
```

Because the assignment of `1` to `(0,0)` would violate the derived constraint that `(0,0)` must be assigned `0`.

We define a convenience function `reduceCs` that a set of constraints `cs` and a placement `p` and returns a optional new set of constraints:

```
let
  out = apply reduceP(_,p) to each member of cs and filter to remove any instances of EMPTY_CONSTRAINT

reduceCs(cs,p) = Err("At least one constraint was violated by the placement."), if out contains None
reduceCs(cs,p) = Ok(out), otherwise
```

### Placements and Boards
In the context of a board along with a set of constraints, a placement is _valid_ if it is legal, and satisfies all of the constraints of the game.  It is _invalid_ otherwise.

We define a function `reduceB` that accepts a board `p` and a placement as arguments and returns a new board:

```
reduceB(b,p) = Ok(Board(b.points - p.points)), if p.points() is a subset of b.points
reduceB(b,p) = Err("Placement {} has at least one point outside of the board.", p), otherwise
```

### Placements and Games
We define a function `play` that accepts a game and a placement as arguments and returns an Result<Game> game, as follows:

```
play(Game(B,P,Cs),p) =
  let
    newB = reduceB(B,p)
    newPs = removeOne(P,p.piece)
    newCs = reduceCs(Cs,p)

  return
    Ok(Game(newB.unwrap(),newPs.unwrap(),newCs.unwrap())), if newB.is_some() && newPs.is_some() && newCs.is_some()
    Err("Unwinnable game."), otherwise
```

### Legal and Illegal Placement Examples
Consider the following board `b` shaped like an "L":

```
b = Board({(0,0),(0,1),(0,2),(1,2)})
```

The placement `Placement((4,4),(0,0),North)` is legal, as is `Placement((3,2),(0,2),East)`.

The placement `Placement((3,3),(0,1),North)` is illegal because the set of points in the board that would result from removing the points in the placement is `{(0,0),(1,2)}`, which is invalid.

### Valid and Invalid Placement Examples
Consider the same board as above together with the set of constraints:

```
{
  AllSame(Some(3), {(0,0)}),
  AllDifferent({}, {(0,2),(1,2)})
}
```

The placement `Placement((3,5),(0,0),North)` is both legal and valid because it assigns `3` to `(0,0)` and satisfies both constraints.  The placement `Placement((3,3),(0,2),East)` is legal but invalid because it does not satisfy the second constraint.

# Playing a Game
We model gameplay as a backtracking algorithm that uses recursion to traverse the space of possible placements of the pieces.

The recursive step accepts a `g=Game(b, pp, c)` and `play_out=Vec<Placement>` as arguments returns `Result<Vec<Placement>>`, as follows:

- If `g` is `WON_GAME`, return `Ok(play_out)`.
- If not, identify the upper-most, left-most point in the board `b`, call it `b0`.
- We now try placing the pieces in different directions, as follows.
  - We compute a set `unique_pieces` that contains the unique pieces in `pp`.
  - For each piece `p` in `unique_pieces`, we use the `play` function to try to place the piece in each of the four directions (omitting `South` and `West` for doubletons) at the point `b0`.  For example, for the `North` direction, we call `play(g,q)` where `q=Placement(p,b0,North)`.  If the call returns `Ok(g2)`, invoke the recursive step with `g2` and `play_out ++ q`.  If the call returns `Err`, continue to the next direction, if any, or to the next piece in `unique_pieces`, if any.  If we've tried all directions and all of the pieces without success, return `Err("No valid placements.")`.

# Textual Input

The game is specified by textual input read from a file that consists of a board, a collection of pieces, and a collection of constraints.

- The first line of the file may be a comment line starting with `//`.
- The rest of the file contains the groups for board, pieces, and constraints in that order.
- The board is specified in a section headed by the line "board:", followed by a potentially empty sequence of lines, and terminated by a blank line.  Each line in the sequence consists of either spaces or an "#" characters.  To convert these lines into a collection coordinates, the first row is `y=0` with successive rows being `y=1`, etc.; and the left-most edge is `x=0` with successive characters being `x=1`, etc.  A `#` character at a position `(x,y)` means that point part of the board.
- The special board `EMPTY_BOARD` is inputted by omitting any lines after the "board:" line and before the separating blank line.
- The pieces are specified on a single line in a section headed by the line "pieces:" and then trailed by a blank line. The line specifying the pieces is a comma-separated list of two-character representations of a piece where each character is a digit in`[0,6]`.
- The constraints are a sequence of lines headed by the line "constraints:" and terminated by a blank line. Each line is of the form `<type> <argument?> {<list of points>}` where `<type>` is a constraint type and `<list of points>` is a comma-separated sequence of ordered pairs of values.

Any deviation from this format is an error, and an informative error message should be returned to the user.  After loading the game, the game should be checked for validity with an informative error message returned if not.

## Example Game Representation
An example game (from the "hard" puzzle on 2025-10-12); this is also in the file `examples/hard_2025-10-12.txt`.  See the `examples` directory for more examples from the Times.

```
// NYTimes Hard 2025-10-12
board:
   #
####
####
####
####
 #

pieces:
06,26,24,64,40,45,51,44,53

constraints:
Exactly 4 {(0,1)}
AllDifferent {} {(1,1),(0,2),(1,2),(2,2)}
Exactly 1 {(2,1)}
AllSame None {(3,0),(3,1),(3,2)}
Exactly 2 {(0,3)}
Exactly 0 {(1,3)}
Exactly 2 {(2,3)}
Exactly 5 {(3,3)}
AllDifferent {} {(0,4),(1,4),(2,4)}
Exactly 3 {(3,4)}
```

The board for the above would be:

```
{
                   (3,0),
 (0,1),(1,1),(2,1),(3,1),
 (0,2),(1,2),(2,2),(3,2),
 (0,3),(1,3),(2,3),(3,3),
 (0,4),(1,4),(2,4),(3,4),
       (1,5)
}
```

And the pieces would be `(0,6),(2,6),(2,4),(6,4),(4,0),(4,5),(5,1),(4,4),(5,3)`.

And the constraints would be:

```
{
  Exactly(4,{(0,1)}),
  AllDifferent({},[(1,1),(0,2),(1,2),(2,2)]),
  Exactly(1,{(2,1)}),
  AllSame(None, {(3,0),(3,1),(3,2)}),
  Exactly(2,{(0,3)}),
  Exactly(0,{(1,3)}),
  Exactly(2,{(2,3)}),
  Exactly(5,{(3,3)}),
  AllDifferent({},[(0,4),(1,4),(2,4)]),
  Exactly(3,{(3,4)}),
}
```
