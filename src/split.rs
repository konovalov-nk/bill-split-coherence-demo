//! Even bill splitting with an explicit remainder policy.
//!
//! Specs: `SPEC-bill-split`, `SPEC-remainder-assignment`.

use std::fmt;

/// Errors produced by a split request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitError {
    /// A split needs at least one participant.
    ZeroParticipants,
    /// Negative totals are outside this domain.
    NegativeTotal,
}

impl fmt::Display for SplitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            Self::ZeroParticipants => "split needs at least one participant",
            Self::NegativeTotal => "split total must not be negative",
        };
        f.write_str(msg)
    }
}

impl std::error::Error for SplitError {}

/// One participant's share.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Allocation {
    pub participant: usize,
    pub cents: i64,
}

/// Result of an even split, including the explanation of where cents went.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SplitResult {
    pub total_cents: i64,
    pub base_cents: i64,
    pub remainder_cents: usize,
    /// Participants that received one extra cent, in assignment order.
    pub remainder_recipients: Vec<usize>,
    pub allocations: Vec<Allocation>,
}

/// Split `total_cents` across `participants` as evenly as possible.
///
/// Every participant gets `total / n`. The `total % n` leftover cents are
/// assigned one per participant using a deterministic order derived from `seed`:
/// the same request is replayable, and randomness is an explicit input.
pub fn split_evenly(
    total_cents: i64,
    participants: usize,
    seed: u64,
) -> Result<SplitResult, SplitError> {
    if participants == 0 {
        return Err(SplitError::ZeroParticipants);
    }
    if total_cents < 0 {
        return Err(SplitError::NegativeTotal);
    }

    let n = participants as i64;
    let base_cents = total_cents / n;
    let remainder_cents = (total_cents % n) as usize;

    let order = seeded_order(participants, seed);
    let remainder_recipients: Vec<usize> = order[..remainder_cents].to_vec();

    let mut allocations: Vec<Allocation> = (0..participants)
        .map(|participant| Allocation {
            participant,
            cents: base_cents,
        })
        .collect();

    for &participant in &remainder_recipients {
        allocations[participant].cents += 1;
    }

    Ok(SplitResult {
        total_cents,
        base_cents,
        remainder_cents,
        remainder_recipients,
        allocations,
    })
}

/// Deterministic Fisher-Yates order over `0..n`, seeded by the caller.
fn seeded_order(n: usize, seed: u64) -> Vec<usize> {
    let mut order: Vec<usize> = (0..n).collect();
    let mut state = seed ^ 0x9E37_79B9_7F4A_7C15;
    if state == 0 {
        state = 1;
    }
    for i in (1..n).rev() {
        state = xorshift64(state);
        let j = (state % (i as u64 + 1)) as usize;
        order.swap(i, j);
    }
    order
}

fn xorshift64(mut x: u64) -> u64 {
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    x
}
