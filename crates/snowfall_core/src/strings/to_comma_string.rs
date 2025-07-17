/// Formats a string as if it were a number and adds commas
/// in an en-us style (e.g. 1000000 -> 1,000,000).
pub fn to_comma_string<T>(n: T) -> String
where
    T: std::fmt::Display,
{
    let s = n.to_string();
    let (neg, s) = if let Some(rest) = s.strip_prefix('-') {
        (true, rest)
    } else {
        (false, s.as_str())
    };

    let mut parts = s.splitn(2, '.');
    let int_part = parts.next().unwrap();
    let frac_part = parts.next();

    let mut res = String::new();
    let chars: Vec<_> = int_part.chars().collect();
    let len = chars.len();
    for (i, c) in chars.iter().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            res.push(',');
        }
        res.push(*c);
    }

    if let Some(frac) = frac_part {
        res.push('.');
        res.push_str(frac);
    }

    if neg { format!("-{}", res) } else { res }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_small_numbers() {
        assert_eq!(to_comma_string(0), "0");
        assert_eq!(to_comma_string(5), "5");
        assert_eq!(to_comma_string(12), "12");
        assert_eq!(to_comma_string(999), "999");
    }

    #[test]
    fn test_thousands() {
        assert_eq!(to_comma_string(1000), "1,000");
        assert_eq!(to_comma_string(1234), "1,234");
        assert_eq!(to_comma_string(9999), "9,999");
    }

    #[test]
    fn test_millions() {
        assert_eq!(to_comma_string(1000000), "1,000,000");
        assert_eq!(to_comma_string(1234567), "1,234,567");
    }

    #[test]
    fn test_large_numbers() {
        assert_eq!(to_comma_string(9876543210u64), "9,876,543,210");
    }

    #[test]
    fn test_negative_numbers() {
        assert_eq!(to_comma_string(-1000), "-1,000");
        assert_eq!(to_comma_string(-1234567), "-1,234,567");
    }

    #[test]
    fn test_string_input() {
        assert_eq!(to_comma_string("1234567"), "1,234,567");
    }

    #[test]
    fn test_decimals() {
        assert_eq!(to_comma_string(0.1), "0.1");
        assert_eq!(to_comma_string(-0.1), "-0.1");
        assert_eq!(to_comma_string(1005.2), "1,005.2");
        assert_eq!(to_comma_string(-1005.2), "-1,005.2");
    }
}
