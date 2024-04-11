pub fn choose(n: u32, k: u32) -> u128 {
    let n = n as u128;
    let k = k as u128;
    fn factorial(n: u128) -> u128 {
        (1..=n).product()
    }
    factorial(n) / factorial(n - k) / factorial(k)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn choose_test() {
        let data = [
            (20, 4, 4845),
            (1, 1, 1),
            (1, 0, 1),
            (23, 4, 8855),
            (6, 3, 20),
            (18, 7, 31824),
        ];
        for (n, k, expected) in data {
            assert_eq!(choose(n, k), expected);
        }
    }
}
