//! Tunable search parameters for SPSA / OpenBench tuning.
//!
//!Mirrors the `tunable_params!` macro from the example snippet.
//! When compiled with `--features tuning`, each parameter is an `AtomicI32` that can be
//! changed at runtime via UCI `setoption name <param> value <val>`.
//! Without the feature, each function is a `const`-folded inline returning the default.
//!
//! # OpenBench SPSA workflow
//! 1. Build with tuning: `cargo build --release --features tuning`
//! 2. Generate SPSA input: `./target/release/Dual --spsa`  or `print_params_ob` via UCI `print_params_ob`
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
// Ranges are intentionally wide but plausible; tighten them after a few test tunes.
// `spsa = false` => UCI-tunable but hidden from SPSA `print_params_ob` (poisoned /
// tiny integer depths). Set to true if you know C_end is safe (>0.5) and you want
// it in the tune.
// ---------------------------------------------------------------------------
#[rustfmt::skip]
tunable_params! {
    // -----------------------------------------------------------------------
    // Reverse Futility Pruning (RFP) — quadratic: margin = a*d^2 + b*d + c - improving*imp
    // Original: 80*(d - improving) => a=0, b=80, c=0, imp=80
    // -----------------------------------------------------------------------
    rfp_max_depth                = 6,   4..=12,             true;
    rfp_a                        = 0,   -10..=20,           true;
    rfp_b                        = 80,  20..=150,           true;
    rfp_c                        = 0,   -50..=50,           true;
    rfp_improving                = 80,  -10..=150,          true;

    // -----------------------------------------------------------------------
    // Razoring — quadratic: threshold = a*d^2 + b*d + c  (eval < alpha - threshold)
    // Original: 200 + 100*d^2 => a=100, b=0, c=200
    // -----------------------------------------------------------------------
    razor_a                      = 100, -100..=300,         true;
    razor_b                      = 0,   -100..=100,         true;
    razor_c                      = 200, -100..=400,         true;

    // -----------------------------------------------------------------------
    // Futility Pruning (quiet) — quadratic: bonus = a*d^2 + b*d + c
    // Original: 80*d => a=0, b=80, c=0
    // -----------------------------------------------------------------------
    fp_max_depth                 = 5,   4..=12,             true;
    fp_a                         = 0,   -10..=20,           true;
    fp_b                         = 80,  20..=150,           true;
    fp_c                         = 0,   -50..=50,           true;

    // -----------------------------------------------------------------------
    // SEE pruning — quadratic: threshold = a*d^2 + b*d + c  (negative)
    // Original: -120 -50*d => a=0, b=-50, c=-120
    // -----------------------------------------------------------------------
    see_a                        = 0,   -10..=10,           true;
    see_b                        = -50, -100..=-10,         true;
    see_c                        = -120,-250..=0,           true;
    qs_see_threshold             = 0,   -50..=50,           true;
    mp_see_threshold             = 0,   -100..=100,         true; // movepicker bad-noisy SEE cut (was 0)

    // SEE piece values
    see_pawn                     = 100, 50..=200,           true;
    see_knight                   = 300, 200..=500,          true;
    see_bishop                   = 300, 200..=500,          true;
    see_rook                     = 500, 300..=700,          true;
    see_queen                    = 900, 700..=1200,         true;

    // -----------------------------------------------------------------------
    // Null Move Pruning — reduction = base + depth / divisor
    // -----------------------------------------------------------------------
    nmp_min_depth                = 3,   2..=5,              true;
    nmp_base                     = 2,   0..=4,              true;
    nmp_divisor                  = 4,   2..=8,              true;

    // -----------------------------------------------------------------------
    // Late Move Pruning — after lmp_base + lmp_scale*d^2 quiets, skip
    // -----------------------------------------------------------------------
    lmp_base                     = 3,   0..=6,              true;
    lmp_scale                    = 1,   1..=3,              true;

    // -----------------------------------------------------------------------
    // History bonus — base = scale*d + offset, then separate float multipliers
    // float multipliers are scaled x100: 100 = 1.0, 0..200 => 0.0..2.0
    // -----------------------------------------------------------------------
    hist_bonus_scale             = 300, 150..=500,          true;
    hist_bonus_offset            = -250, -500..=0,          true;
    hist_beta_mult               = 100, 0..=200,            true; // beta cutoff
    hist_alpha_mult              = 100, 0..=200,            true; // alpha raise (improve)
    hist_malus_mult              = 100, 0..=200,            true; // penalty for quiets before cutoff

    // -----------------------------------------------------------------------
    // LMR — reduction = (0.99 + ln(d)*ln(m)/3.14)*1024 - history/div
    // 0.99 and 3.14 are scaled x100 to keep int tuning: 99 and 314
    // -----------------------------------------------------------------------
    lmr_min_depth                = 3,   2..=5,              true;
    lmr_min_moves                = 2,   1..=4,              true;
    lmr_hist_div                 = 8,   2..=16,             true;
    lmr_base                     = 99,  50..=150,           true; // 0.99*100
    lmr_div                      = 314, 200..=500,          true; // 3.14*100

    // -----------------------------------------------------------------------
    // TT replacement
    // -----------------------------------------------------------------------
    tt_age_weight                = 6,   0..=12,             true; // age diff * weight
    tt_exact_bonus               = 1,   0..=4,              true; // Exact gets +1 depth

    // -----------------------------------------------------------------------
    // Time management — StopCondition
    // -----------------------------------------------------------------------
    tm_hard_percent              = 75,  50..=95,            true; // hard limit = time * percent/100
    tm_alloc_div                 = 15,  8..=30,             true; // allocation = time/alloc_div + inc*inc_scale/100
    tm_soft_div                  = 3,   2..=6,              true; // soft = allocation / soft_div
    tm_inc_scale                 = 100, 0..=200,            true; // inc multiplier x100 (100=1.0)

    // -----------------------------------------------------------------------
    // History clamp
    // -----------------------------------------------------------------------
    max_history                  = 16384, 8192..=32768,     false;

    // -----------------------------------------------------------------------
    // LMP / Aspiration max tries (poisoned small ints -> spsa false)
    // -----------------------------------------------------------------------
    lmp_max_depth                = 12,  6..=20,             true; // LMP only if depth <= this (large = almost always). Set high to keep current behavior.
    asp_max_tries                = 3,   2..=6,              true; // aspiration re-searches before fallback

    // -----------------------------------------------------------------------
    // Aspiration window
    // -----------------------------------------------------------------------
    asp_delta                    = 50,  10..=100,           true;
    asp_mult                     = 2,   2..=4,              true;
}
