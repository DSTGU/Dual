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
pub enum Granularity {
    Disabled,
    Fine,
    Coarse,
}


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
                match $spsa {
                    Granularity::Disabled => {}

                    Granularity::Fine => {
                        let step = ($max - $min) as f64 / 50.0;

                        println!(
                            "{}, int, {}.0, {}.0, {}.0, {}, 0.002",
                            stringify!($name),
                            $name(),
                            $min,
                            $max,
                            step,
                        );
                    }

                    Granularity::Coarse => {
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
    rfp_max_depth                = 9,   4..=12,             Granularity::Disabled;
    rfp_a                        = 5,   -10..=20,           Granularity::Fine; // TODO: Quantize
    rfp_b                        = 52,  20..=150,           Granularity::Fine;
    rfp_c                        = 0,   -50..=50,           Granularity::Coarse;
    rfp_improving                = 36,  -10..=150,          Granularity::Coarse;

    // -----------------------------------------------------------------------
    // Razoring — quadratic: threshold = a*d^2 + b*d + c  (eval < alpha - threshold)
    // Original: 200 + 100*d^2 => a=100, b=0, c=200
    // -----------------------------------------------------------------------
    razor_a                      = 124, -100..=300,         Granularity::Coarse;
    razor_b                      = -1,  -100..=100,         Granularity::Fine;
    razor_c                      = 23,  -100..=400,         Granularity::Coarse;

    // -----------------------------------------------------------------------
    // Futility Pruning (quiet) — quadratic: bonus = a*d^2 + b*d + c
    // Original: 80*d => a=0, b=80, c=0
    // -----------------------------------------------------------------------
    fp_max_depth                 = 6,    4..=12,             Granularity::Disabled;
    fp_a                         = -4,   -10..=20,           Granularity::Fine;
    fp_b                         = 121,  20..=150,           Granularity::Fine;
    fp_c                         = -6,  -50..=50,           Granularity::Coarse;

    // -----------------------------------------------------------------------
    // SEE pruning — quadratic: threshold = a*d^2 + b*d + c  (negative)
    // Original: -120 -50*d => a=0, b=-50, c=-120
    // -----------------------------------------------------------------------
    see_a                        = -4,  -10..=10,           Granularity::Coarse;
    see_b                        = -39, -100..=-10,         Granularity::Coarse;
    see_c                        = -67, -250..=0,           Granularity::Coarse;
    qs_see_threshold             = -12,   -50..=50,         Granularity::Coarse;
    mp_see_threshold             = -29,   -100..=100,       Granularity::Coarse; // movepicker bad-noisy SEE cut (was 0)

    // SEE piece values
    see_pawn                     = 131, 50..=200,           Granularity::Coarse;
    see_knight                   = 237, 200..=500,          Granularity::Coarse;
    see_bishop                   = 352, 200..=500,          Granularity::Coarse;
    see_rook                     = 607, 300..=700,          Granularity::Coarse;
    see_queen                    = 763, 700..=1200,         Granularity::Coarse;

    // -----------------------------------------------------------------------
    // Null Move Pruning — reduction = base + depth / divisor
    // -----------------------------------------------------------------------
    nmp_min_depth                = 3,   2..=5,              Granularity::Disabled;
    nmp_base                     = 3,   0..=4,              Granularity::Disabled; // TODO: Quantize
    nmp_divisor                  = 4,   2..=8,              Granularity::Disabled; // TODO: Quantize

    // -----------------------------------------------------------------------
    // Late Move Pruning — after lmp_base + lmp_scale*d^2 quiets, skip
    // -----------------------------------------------------------------------
    lmp_base                     = 2,   0..=6,              Granularity::Disabled; // TODO: Quantize
    lmp_scale                    = 1,   1..=3,              Granularity::Disabled; // TODO: Quantize

    // -----------------------------------------------------------------------
    // History bonus — base = scale*d + offset, then separate float multipliers
    // float multipliers are scaled x100: 100 = 1.0, 0..200 => 0.0..2.0
    // -----------------------------------------------------------------------
    hist_bonus_scale             = 385, 150..=500,          Granularity::Fine;
    hist_bonus_offset            = -194, -500..=0,          Granularity::Fine;
    hist_beta_mult               = 107, 0..=200,             Granularity::Coarse; // beta cutoff
    hist_alpha_mult              = 73, 0..=200,             Granularity::Coarse; // alpha raise (improve)
    hist_malus_mult              = 91, 0..=200,            Granularity::Coarse; // penalty for quiets before cutoff

    // -----------------------------------------------------------------------
    // LMR — reduction = (0.99 + ln(d)*ln(m)/3.14)*1024 - history/div
    // 0.99 and 3.14 are scaled x100 to keep int tuning: 99 and 314
    // -----------------------------------------------------------------------
    lmr_min_depth                = 3,   2..=5,              Granularity::Disabled;
    lmr_min_moves                = 2,   1..=4,              Granularity::Disabled;
    lmr_hist_div                 = 10,   2..=16,            Granularity::Disabled;
    lmr_base                     = 66,  50..=150,           Granularity::Fine; // 0.99*100
    lmr_div                      = 226, 200..=500,          Granularity::Fine; // 3.14*100

    // -----------------------------------------------------------------------
    // TT replacement
    // -----------------------------------------------------------------------
    tt_age_weight                = 8,   0..=12,             Granularity::Disabled; // age diff * weight
    tt_exact_bonus               = 1,   0..=4,              Granularity::Disabled; // Exact gets +1 depth

    // -----------------------------------------------------------------------
    // Time management — StopCondition
    // -----------------------------------------------------------------------
    tm_hard_percent              = 72,  50..=95,            Granularity::Fine; // hard limit = time * percent/100
    tm_alloc_div                 = 12,  8..=30,             Granularity::Fine; // allocation = time/alloc_div + inc*inc_scale/100
    tm_soft_div                  = 3,   2..=6,              Granularity::Disabled; // soft = allocation / soft_div // TODO: Quantize
    tm_inc_scale                 = 141, 0..=200,            Granularity::Fine; // inc multiplier x100 (100=1.0)

    // -----------------------------------------------------------------------
    // History clamp
    // -----------------------------------------------------------------------
    max_history                  = 16384, 8192..=32768,     Granularity::Disabled;

    // -----------------------------------------------------------------------
    // LMP / Aspiration max tries (poisoned small ints -> spsa false)
    // -----------------------------------------------------------------------
    lmp_max_depth                = 13,  6..=20,             Granularity::Disabled; // LMP only if depth <= this (large = almost always). Set high to keep current behavior.
    asp_max_tries                = 3,   2..=6,              Granularity::Disabled; // aspiration re-searches before fallback

    // -----------------------------------------------------------------------
    // Aspiration window
    // -----------------------------------------------------------------------
    asp_delta                    = 65,  10..=100,           Granularity::Coarse;
    asp_mult                     = 2,   2..=4,              Granularity::Disabled; // TODO: Quantize

    // -----------------------------------------------------------------------
    // IIR
    // -----------------------------------------------------------------------
    iir_depth                    = 5,   2..=10,              Granularity::Disabled;
}
