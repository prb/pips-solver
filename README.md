# Pips Solver
Human-specified, AI-implemented solver for the NY Times Pips game.  The specification is in [strategy.md](strategy.md), and I'm experimenting with different AI models and what it's like to work with them.  I used the [Zed](https://zed.dev) editor for authoring.

My favorite puzzle is 2025-10-15 "hard":

```
411
4 3  6
551  0
4    0
2    0
 520
 4 1  20
 630  2
 6    3
 2   33
```

## Gemini 2.5 Pro
I used Gemini 2.5 Pro to make a first pass from within the Zed editor, interacting with the AI to build a working solver; net net, it was certainly faster than writing it on my own.  After making passes with Claude and Codex, I used Gemini 2.5 Pro again (but from the CLI this time) for a second pass after the refinements to the specification obtained through the interaction with the other models.

For the first pass, Gemini didn't do a great job following the instructions in the specificaiton,

## Claude Sonnet 4.5 + Code
I used Claude Sonnet 4.5 via the Claude Code CLI (at the cost of around 8% of a week's usage limit for a Pro plan) for another pass, and it did a superlative job.  It found both subtle and unsubtle issues in the specification, and it produced well-tested code in the format and style that I requested.

Interestingly, the solver was 4-5x as fast as the Gemini solver once it identified (and implemented) a performance optimization by reading the Gemini code.

## Codex 5
I used GPT Codex 5 (at the cost of around 500k total tokens) for another pass, and it also did a great job, including finding some subtle and unsubtle issues with the specification that both Claude and Gemini had missed.  Interestingly, when I asked it to critique the other two implementations, it complained that they correctly implemented the specification in a way that Codex had missed!  (Codex admitted the error once prompted.)  Codex produced well-tested code in the format and style that I requested.  The Codex solver is about twice as fast as the Gemini solver (and thus half as fast as the Claude solver).

Developed last, the Codex solver is the most feature complete and correct.

I did get Codex to add a simple post-solution board display function, e.g.:

```
Board:
  44
 3366526
 6 454243
66 255200
   11  00
   11  21
```

And I got Codex to write a converter from the NYTimes JSON format to the textual format used in this project.

## Cost Considerations
On my first pass using Gemini from Zed, I provisioned an API key in Google Cloud, assigned it to the non-free tier, and that resulted in a cost of around $35 for the work.  That's great compared to the cost of human labor, but it's nearly double the monthly $20 subscription costs for either Claude or Codex.  For the second pass using Gemini from the commandline, I authenticated to Google and used the Gemini subscription from my Google account.

## Acknowledgments / References
Discovering and reviewing the code for another Pips solving project,[pips](https://github.com/ematth/pips), I discovered that a JSON representation of the games is downloadable from the NY Times API; this helped to bulk up the set of examples.  The 2025-09-15 "hard" game is the most interesting because of the large `Exactly` constraint.

## Future
Things that I might tinker with further could include parallelization with Rayon or similar, trying out AlgorithmX/Dancing Links, prettier output, and accepting a screenshot of a game as input.

## Examples
To experiment with the solvers, there are a number of examples (in [examples/](examples)) pulled from the NYTimes.  Running them looks something like this for the Gemini solver:

```
> cargo run --release -- ../examples/hard_2025-10-14_puzzle.txt
  Compiling anyhow v1.0.100
   Compiling memchr v2.7.6
   Compiling nom v8.0.0
   Compiling pips-solver v0.1.0 (/Users/paulbrown/Code/pips-solver)
    Finished `release` profile [optimized] target(s) in 2.03s
     Running `target/release/pips-solver examples/hard_2025-10-14_puzzle.txt`
Pips Solver
Loading game from: ../examples/hard_2025-10-14_puzzle.txt
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

Note that running with the `--release` flag is critical for larger puzzles, or the 60s timeout will likely be overrun.

Visually, the solver is operating on this puzzle:

![unsolved puzzle](/images/IMG_8091.jpeg)

And producing this solution:

![solved puzzle](/images/IMG_8092.jpeg)
