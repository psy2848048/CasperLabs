use crate::math::sqrt_for_u512;

use types::U512;

pub fn pop_score_calculation(total_delegated: &U512, validator_delegated_amount: &U512) -> U512 {
    // Currenrly running in PoS.
    // Profession factor will be added soon
    let profession_factor = U512::from(1);

    let x = *validator_delegated_amount * U512::from(100) / *total_delegated;

    let score = if x <= U512::from(15) {
        // y = 1000x
        *validator_delegated_amount * U512::from(100_000) / *total_delegated
    } else {
        // y = 1000 * sqrt(30x - 225)
        //   = sqrt(1_000_000 * 30 * 100 * val_delegation / total_delegated - 225_000_000)
        sqrt_for_u512(
            *validator_delegated_amount * U512::from(3_000_000_000_u64) / *total_delegated
                - U512::from(225_000_000),
        )
    };

    score * profession_factor
}
