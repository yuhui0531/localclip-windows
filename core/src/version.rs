pub fn marketing(value: &str) -> &str {
    value
        .trim_start_matches(['v', 'V'])
        .split(['-', '+'])
        .next()
        .unwrap_or("")
}

pub fn is_newer(candidate: &str, current: &str) -> bool {
    let parts = |value: &str| -> Vec<i64> {
        marketing(value)
            .split('.')
            .filter(|s| !s.is_empty())
            .map(|s| s.parse().unwrap_or(0))
            .collect()
    };
    let candidate = parts(candidate);
    let current = parts(current);
    for index in 0..candidate.len().max(current.len()) {
        match candidate
            .get(index)
            .unwrap_or(&0)
            .cmp(current.get(index).unwrap_or(&0))
        {
            std::cmp::Ordering::Greater => return true,
            std::cmp::Ordering::Less => return false,
            std::cmp::Ordering::Equal => {}
        }
    }
    false
}
