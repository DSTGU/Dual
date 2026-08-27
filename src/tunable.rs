//! Tunable search parameters for SPSA / OpenBench tuning.
//!
//!mirrors the `tunable_params!` macro from the example snippet.
//! When compiled with `--features tuning`, each parameter is an `AtomicI32` that can be
//! changed at runtime via UCI `setoption name <param> value <val>`.
//! Without the feature, each function is a `const`-folded inline returning the default.
//!
//! # OpenBench SPSA workflow
//! 1. Build with tuning: `cargo build --release --features tuning`
//! 2. Generate SPSA input: `./target/release/Dual --spsa`  or `print_params_ob` via UCI `print_params_ob` (if wired)
//!    which prints lines like:
//!    `rfp_scale, int, 80.0, 30.0, 150.0, 6.0, 0.002`
//! 3. Paste that into OpenBench "SPSA Input" when creating a tune.
//! 4. OpenBench will run games with `setoption name rfp_scale value <perturbed>` pairs.
//!
//! C_end is auto-computed as `(max - min) / 20` (same as example). R_end is fixed `0.002`.
//! For very small / volatile integer thresholds (depth cutoffs, LMP base, etc.) the
//! `spsa` flag is `false` — they are still UCI-tunable but omitted from SPSA `print_params_ob`
//! to avoid poisoning. Tune them manually or enable the flag deliberately (see article note
//! about C_end >= 0.50 for integers and poisoned parameters).

#[macro_export]
macro_rules! tunable_params {
    ($($name:ident = $val:expr, $min:literal ..= $max:literal, $spsa:expr;)*) => {
        #[cfg(feature = "tuning")]
        use std::sync::atomic::Ordering;

        /// Print UCI `option name ...` lines for every tunable param.
        #[cfg(feature = "tuning")]
        pub fn list_params() {
            $(
                println!(
                    "option name {} type spin default {} min {} max {}",
                    stringify!($name),
                    $name(),
                    $min,
                    $max,
                );
            )*
        }

        /// Set a param by name (UCI `setoption name X value Y`).
        #[cfg(feature = "tuning")]
        pub fn set_param(name: &str, val: i32) {
            match name {
                $(
                    stringify!($name) => vals::$name.store(val, Ordering::Relaxed),
                )*
                _ => println!("info error unknown option {}", name),
            }
        }

        /// Print OpenBench SPSA input lines for params marked `spsa = true`.
        /// Format per line: `name, int, cur.0, min.0, max.0, step.0, 0.002`
        /// where step = (max - min) / 20.0
        #[cfg(feature = "tuning")]
        pub fn print_params_ob() {
            $(
                if $spsa {
                    let step = ($max - $min) as f64 / 20.0;
                    println!(
                        "{}, int, {}.0, {}.0, {}.0, {}, 0.002",
                        stringify!($name),
                        $name(),
                        $min,
                        $max,
                        step,
                    );
                }
            )*
        }

        #[cfg(feature = "tuning")]
        mod vals {
            use std::sync::atomic::AtomicI32;
            $(
            #[allow(non_upper_case_globals)]
            pub static $name: AtomicI32 = AtomicI32::new($val);
            )*
        }

        $(
        #[cfg(feature = "tuning")]
        #[inline]
        pub fn $name() -> i32 {
            vals::$name.load(Ordering::Relaxed)
        }

        #[cfg(not(feature = "tuning"))]
        #[inline]
        pub fn $name() -> i32 {
            $val
        }
        )*
    };
}

// ---------------------------------------------------------------------------
// Actual search parameters for Dual.
// Every hardcoded literal from `search.rs` that is interesting to tune is
// exposed here.  Ranges are intentionally wide but plausible; tighten them after
// a few test tunes.
// `spsa = false` => UCI-tunable but hidden from SPSA `print_params_ob` (poisoned /
// tiny integer depths). Set to true if you know C_end is safe (>0.5) and you want
// it in the tune.
// ---------------------------------------------------------------------------
#[rustfmt::skip]
tunable_params! {
    // Reverse Futility Pruning (RFP) — depth <= rfp_max_depth, margin = scale * (depth - improving)
    rfp_max_depth                = 6,   3..=9,              false;
    rfp_scale                    = 80,  30..=150,           true;

    // Razoring — if eval < alpha - razor_base - razor_scale * depth^2  => qsearch
    razor_base                   = 200, 0..=400,            true;
    razor_scale                  = 100, 30..=300,           true;

    // Null Move Pruning — depth >= nmp_min_depth, static_eval > beta, reduction = nmp_base + depth / nmp_divisor
    nmp_min_depth                = 3,   2..=5,              false;
    nmp_base                     = 2,   0..=4,              false;
    nmp_divisor                  = 4,   2..=8,              false;

    // Futility Pruning (quiet) — depth <= fp_max_depth, eval + fp_scale * depth <= alpha
    fp_max_depth                 = 5,   3..=8,              false;
    fp_scale                     = 80,  20..=150,           true;

    // Late Move Pruning — after lmp_base + depth^2 quiets, skip remaining quiets
    lmp_base                     = 3,   0..=6,              true;
    // lmp_scale multiplies depth^2 (1 == original: base + 1*depth^2). Keep small range.
    lmp_scale                    = 1,   1..=3,              false;

    // SEE pruning — threshold = see_base + see_scale * depth  (both negative => threshold negative)
    // default -120 -50*depth
    see_base                     = -120, -250..=0,          true;
    see_scale                    = -50,  -100..=-10,        true;
    // QS SEE threshold (was hardcoded 0)
    qs_see_threshold             = 0,   -50..=50,           true;

    // History bonus — bonus = hist_scale * depth + hist_offset  (offset negative in original)
    hist_bonus_scale             = 300, 150..=500,          true;
    hist_bonus_offset            = -250, -500..=0,          true;

    // LMR — reduction = reduce_lmr_by(depth,moves) - history / lmr_hist_div  (then /1024 clamped)
    lmr_min_depth                = 3,   2..=5,              false;
    lmr_min_moves                = 2,   1..=4,              false;
    lmr_hist_div                 = 8,   2..=16,             true;

    // Aspiration window — initial delta, doubling on fail (original 50, *2 each retry, up to 3 tries)
    asp_delta                    = 50,  10..=100,           true;
    // asp_fails not directly tuned; keep tries =3 hardcoded. If you want to tune multiplier, use asp_mult.
    asp_mult                     = 2,   2..=3,              false;
}
