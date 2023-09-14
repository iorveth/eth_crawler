use std::collections::BTreeSet;

// Retrieves block ranges for unfetched transactions
pub fn get_block_ranges_for_unfetched_transactions(
    fetched_block_numbers_since_block: BTreeSet<u64>,
    starting_block_number: u64,
    current_block_number: u64,
) -> Vec<(u64, u64)> {
    let all_block_numbers_since_block: BTreeSet<u64> =
        (starting_block_number..current_block_number).collect();

    // I assume it should be sorted in ascending order here
    let unfetched_block_numbers_since_block: Vec<u64> = all_block_numbers_since_block
        .difference(&fetched_block_numbers_since_block)
        .cloned()
        .collect();

    let mut ranges = vec![];

    let mut start = unfetched_block_numbers_since_block[0];
    for i in 1..unfetched_block_numbers_since_block.len() {
        let value = unfetched_block_numbers_since_block[i];
        let previous = unfetched_block_numbers_since_block[i - 1];

        if value == previous + 1 {
            continue;
        }

        if start == previous {
            ranges.push((previous, previous));
        } else {
            ranges.push((start, previous));
        }
        start = value;
    }

    // Save last range
    if let Some((_, last)) = ranges.last().cloned() {
        ranges.push((last + 2, current_block_number))
    }

    if ranges.is_empty() {
        ranges.push((
            unfetched_block_numbers_since_block[0],
            unfetched_block_numbers_since_block[unfetched_block_numbers_since_block.len() - 1],
        ));
    }

    ranges
}
