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

pub fn float_range(start: f64, end: f64, step: f64) -> impl Iterator<Item=f64> {
    FloatRange {
        end,
        step,
        current: start,
        reached_end: false,
    }
}

struct FloatRange {
    end: f64,
    step: f64,
    current: f64,
    reached_end: bool
}

impl Iterator for FloatRange {
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        if self.reached_end {
            return None;
        }

        if self.current > self.end {
            self.reached_end = true;
            return Some(self.end);
        }

        let current = self.current;
        self.current += self.step;
        Some(current)
    }
}