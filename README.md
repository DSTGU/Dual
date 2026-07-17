Amateur chess engine

Currently features:
 - Quiescence search
 - History Heuristic
 - Killer Heuristic
 - MVV_LVA
 - Transposition table (move ordering + probing) (TT size unconfigurable ~24MB)
 - PVS
 - LMR
 - NMP
 - Futility + Reverse futility pruning
 - Aspiration windows
 - NNUE

Estimated current rating for dev: ~2950
Estimated current rating for release: 2750
For more info see releases tab.

Todo for 1.0.0:
 - Net improvement and optimisations (fused updates, manual simd)
 - Add actual config including tt
 - Look into QS improvements

Todo for 1.1.0:
 - SEE (move ordering + pruning)
 - LMP
 - Razoring
 - True engine selfplay datagen
 - Experiment with different net architectures (namely: hm, maybe buckets)

Future plans:
 - Test Correction/Continuation/Countermove history
 - Capture history
 - Tuning
 - Some other easy improvements from furypasta
 - Make stronger in general :>

No ranking or tournament results as of now

Thanks to:
Maksym Korzh
Engine Programming discord
jw1912 - Bullet library creator

Bullet was used to train the network used by the engine

---

Copyright (C) 2026 Tomasz Stawowy

This program is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License along with this program.  If not, see <https://www.gnu.org/licenses/>.
