//! Runnable demo for the bill-split specs.
//!
//! `cargo run -- 1000.00 3 42`
//! `cargo run -- 1000.01 3 7`

use std::env;
use std::process::ExitCode;

use coherence_core_db_bootstrap::money::{format_cents, parse_eur_to_cents};
use coherence_core_db_bootstrap::split::split_evenly;

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let amount = args.next().unwrap_or_else(|| "1000.00".to_string());
    let participants = args
        .next()
        .unwrap_or_else(|| "3".to_string())
        .parse::<usize>()
        .unwrap_or(3);
    let seed = args
        .next()
        .unwrap_or_else(|| "42".to_string())
        .parse::<u64>()
        .unwrap_or(42);

    let total_cents = match parse_eur_to_cents(&amount) {
        Ok(cents) => cents,
        Err(err) => {
            eprintln!("cannot parse {amount:?}: {err}");
            return ExitCode::from(2);
        }
    };

    let result = match split_evenly(total_cents, participants, seed) {
        Ok(result) => result,
        Err(err) => {
            eprintln!("cannot split: {err}");
            return ExitCode::from(2);
        }
    };

    println!("total:        {} EUR", format_cents(result.total_cents));
    println!("participants: {}", result.allocations.len());
    println!("base share:   {} EUR", format_cents(result.base_cents));
    println!(
        "remainder:    {} EUR -> {:?}",
        format_cents(result.remainder_cents as i64),
        result.remainder_recipients
    );
    println!("seed:         {seed}");
    println!();
    for allocation in &result.allocations {
        println!(
            "participant {}: {} EUR",
            allocation.participant,
            format_cents(allocation.cents)
        );
    }

    let sum: i64 = result.allocations.iter().map(|a| a.cents).sum();
    println!();
    println!("sum:          {} EUR", format_cents(sum));
    ExitCode::SUCCESS
}
