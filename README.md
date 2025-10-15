# Pips Solver
Human-specified, AI-implemented  solver for the NY Times Pips game.  The specification is in [strategy.md](strategy.md).  The AI used was this initial pass was Gemini 2.5 Pro at a cost of around $10 plus a few hours of human time to write the specification and a few more hours of human time to interact with the AI; net net, it was certainly faster than writing it on my own.  I used the [Zed](https://zed.dev) editor for authoring and to work with the AI.

There are a number of examples (in [examples/](examples) pulled from the NYTimes, and running them looks something like this:

```
> cargo run --release -- examples/hard_2025-10-14_puzzle.txt 
  Compiling anyhow v1.0.100
   Compiling memchr v2.7.6
   Compiling nom v8.0.0
   Compiling pips-solver v0.1.0 (/Users/paulbrown/Code/pips-solver)
    Finished `release` profile [optimized] target(s) in 2.03s
     Running `target/release/pips-solver examples/hard_2025-10-14_puzzle.txt`
Pips Solver
Loading game from: examples/hard_2025-10-14_puzzle.txt
Game loaded successfully!
Solving...

Solution found in 42.96s!
Placements:
  - Piece (6, 6) at (0, 3) -> East
  - Piece (3, 6) at (1, 1) -> South
  - Piece (3, 4) at (2, 0) -> North
  - Piece (4, 6) at (3, 0) -> South
  - Piece (4, 5) at (3, 2) -> East
  - Piece (1, 2) at (3, 3) -> North
  - Piece (1, 1) at (3, 5) -> East
  - Piece (5, 6) at (4, 1) -> West
  - Piece (1, 5) at (4, 3) -> North
  - Piece (2, 4) at (5, 2) -> West
  - Piece (2, 5) at (5, 3) -> West
  - Piece (2, 6) at (6, 1) -> East
  - Piece (0, 4) at (7, 2) -> North
  - Piece (0, 2) at (7, 4) -> South
  - Piece (0, 3) at (8, 2) -> North
  - Piece (0, 1) at (8, 4) -> South
```

And running with the `--release` flag is critical for larger puzzles, or the 60s timeout will likely be overrun.

If you read the specification and then look at the source code, it will become obvious that a some of specification was ignored (e.g., putting the code into separate modules, idiomatic functional style, etc.) as well as defining (and using) incremental unit tests.

