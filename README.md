# Dual

An amateur UCI chess engine written in Rust.

## Features

- **NNUE Evaluation** - a classic (768 -> 256)x2 -> 1 architecture network trained using excellent Bullet library on self generated data
- **Search** - Alpha-beta based search algorithm with additional search feature set:

**General Search features**:
 - Principal variation search
 - Quiescence search
 - Late Move Reductions
 - Aspiration windows
 - Iterative deepening

**Move ordering**:
 - Transposition table
 - MVV_LVA
 - Killer Heuristic
 - History Heuristic
 - SEE (good and bad captures)

Forward pruning:
 - Transposition table
 - Null Move Pruning
 - Futility pruning
 - Reverse futility pruning
 - Static Exchange Evaluation
 - Late Move Pruning
 - Futility + Reverse futility pruning
 - Razoring

## Future plans:

Todo for 1.0.0:
 - Net improvement and optimisations (fused updates, manual simd, hm)
 - Correct tt format (with static eval and buckets)
 - Fix clippy lints
 - Hammer out all(most) performance time sinks correctly
 - Movegen improvements (legal?)
 - IIR
 - Proper move stack with static eval data
 - Improving

Future plans:
 - Correction/Continuation/Countermove history
 - Capture history
 - Tuning
 - Other easy improvements from furypasta
 - Experiment with different net architectures (namely: hm, maybe buckets)
 - Make stronger in general :>

## UCI Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| Hash | spin (1-1024) | 64 | Transposition table size in MB |
| Threads | spin (1-1) | 1 | Thread count option stub |
| SoftNodes | spin (0-1000000000) | 0 | UCI option for giving soft limit to depth search |

## Strength

| Version | Release Date | COPE Bullet | COPE Rapid | CCI VLTC |
|---------|--------------|-------------|------------|----------|
| v0.4.2 | 2026-08-08    |   TBD       |   TBD      | 3085     |
| v0.4.1 | 2026-07-26    | 3041 (#54)  | 3170 (#55) | 2853     |
| v0.4.0 | 2026-07-19    |   ---       |  ---       | 2782     |
| v0.3.2 | 2026-07-06    |  ---        | ---        | 2662     |

## Credits

- **Maksym Korzh** - BCC engine and video series creator
- Engine Programming discord + SF discord
- **jw1912** - Bullet library creator. Bullet has been used for training the nets used by Dual
- Other engines Dual takes inspiration from, including (but not limited to) Reckless, Icarus, Hobbes, Berserk, Stormphrax and Caissa

---

## License

Copyright (C) 2026 Tomasz Stawowy

This program is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License along with this program.  If not, see <https://www.gnu.org/licenses/>.
