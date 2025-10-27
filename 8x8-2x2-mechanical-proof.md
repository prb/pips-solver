# 8x8-2x2 Mechanical Proof

In the article [Polyominoes 101 - The Absolute Basics](https://polyominoes.co.uk/polyominoes/101/index.html), the assertion is made that if you take an 8x8 grid and remove any 2x2 square, the remaining grid can be tiled by using each of the 12 polyominoes (up to chirality) exactly once each.

There are 49 possible 8x8-2x2 grids corresponding to the possible upper left coordinates of the upper left corner of the 2x2 square that is removed.  (As in the other examples in this overall project, we number coordinates starting at `0` with `x` running left to right and `y` running top to bottom.)  The number of cases significantly reduces when considering the possible symmetries of the 8x8-2x2 grid to just 10 with the upper left corner of the 2x2 square being at one of:

```
{(0,0),(1,0),(2,0),(3,0),
       (1,1),(2,1),(3,1),
             (2,2),(3,2),
                   (3,3)}
```

We want to complete a mechanical proof of the assertion above by creating a tiling for each of the 10 cases in the form of an executable program.

The desired output for each of the ten solutions is a board  ASCII pretty-printed style that tags each polyonimo with its letter, e.g.:

```
┌───┬───────────────┬───────┬───┐
│ F │ L   L   L   L │ Z   Z │ Y │
│   └───────┬───┐   │   ┌───┘   │
│ F   F   F │ W │ L │ Z │ Y   Y │
├───┐   ┌───┘   ├───┘   ├───┐   │
│ V │ F │ W   W │  Z  Z │ P │ Y │
│   ├───┘   ┌───┴───┬───┘   │   │
│ V │ W   W │       │ P   P │ Y │
│   └───────┤       │       ├───┤
│ V   V   V │       │ P   P │ T │
├───────┬───┼───────┼───────┘   │
│ U   U │ X │ S   S │ T   T   T │
│   ┌───┘   └───┐   └───────┐   │
│ U │ X   X   X │ S   S   S │ T │
│   └───┐   ┌───┴───────────┴───┤
│ U   U │ X │ I   I   I   I   I │
└───────┴───┴───────────────────┘
```
