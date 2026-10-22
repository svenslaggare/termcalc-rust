pub fn to_base(mut value: i64, base: u32, prefix: &str) -> String {
    const ALPHA_NUMERIC: [char; 36] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z'];

    let mut result = Vec::new();
    let is_negative = value < 0;
    value = value.abs();
    let base = base as i64;

    while value > 0 {
        result.push(ALPHA_NUMERIC[(value % base) as usize]);
        value /= base;
    }

    result.reverse();

    format!("{}{}{}", if is_negative {"-"} else {""}, prefix, String::from_iter(result.into_iter()))
}