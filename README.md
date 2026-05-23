Begginer chess engine

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

Estimated current rating: ~2650 (+-50)
For more info see releases tab.

Future plans:
 - Add actual config
 - Tuning (aspiration window size)
 - Test Correction/Continuation/Countermove history
 - SEE
 - Capture history
 - 50MR
 - Investigate TT improvements (replacement strategy based on age + potential inneficienies + size)

No ranking or tournament results as of now

Thanks to:
Maksym Korzh
Engine Programming discord
jw1912 - Bullet library creator

Bullet was used to train the network used by the engine
