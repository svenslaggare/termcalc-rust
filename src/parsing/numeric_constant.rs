use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct NumericConstant {
    chars: Vec<NumericConstantChar>,
    base: u32
}

impl NumericConstant {
    pub fn new(chars: Vec<NumericConstantChar>, base: u32) -> NumericConstant {
        NumericConstant {
            chars,
            base
        }
    }

    pub fn from_i64(value: i64) -> NumericConstant {
        NumericConstant::from_str(&value.to_string()).unwrap()
    }

    pub fn from_f64(value: f64) -> NumericConstant {
        NumericConstant::from_str(&f64_to_string_full(value)).unwrap()
    }
    
    pub fn iter(&self) -> impl Iterator<Item=NumericConstantChar> {
        self.chars.iter().cloned()
    }
}

fn f64_to_string_full(value: f64) -> String {
    let int_part = value.floor();
    let frac_part = value - int_part;

    if frac_part == 0.0 {
        int_part.to_string() + ".0"
    } else {
        format!("{:.17}", value)
    }
}

impl FromStr for NumericConstant {
    type Err = NumericConstantConstructionError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let mut has_minus_sign = false;
        let mut has_plus_sign = false;
        let mut has_decimal_point = false;
        let mut has_imaginary_unit = false;
        let mut is_complex_mode = false;

        let mut chars = Vec::new();
        for (index, current) in text.chars().enumerate() {
            match current {
                '0' => { chars.push(NumericConstantChar::Zero); },
                '1' => { chars.push(NumericConstantChar::One); },
                '2' => { chars.push(NumericConstantChar::Two); },
                '3' => { chars.push(NumericConstantChar::Three); },
                '4' => { chars.push(NumericConstantChar::Four); },
                '5' => { chars.push(NumericConstantChar::Five); },
                '6' => { chars.push(NumericConstantChar::Six); },
                '7' => { chars.push(NumericConstantChar::Seven); },
                '8' => { chars.push(NumericConstantChar::Eight); },
                '9' => { chars.push(NumericConstantChar::Nine); },
                '-' => {
                    if index > 0 && is_complex_mode {
                        is_complex_mode = true;
                        has_minus_sign = false;
                        has_plus_sign = false;
                        has_decimal_point = false;
                    }

                    if has_minus_sign {
                        return Err(NumericConstantConstructionError::HasMinusSign);
                    }

                    has_minus_sign = true;
                    chars.push(NumericConstantChar::MinusSign);
                }
                '+' => {
                    if index > 0 && !is_complex_mode {
                        is_complex_mode = true;
                        has_minus_sign = false;
                        has_plus_sign = false;
                        has_decimal_point = false;
                    }

                    if has_plus_sign {
                        return Err(NumericConstantConstructionError::HasPlusSign);
                    }

                    has_plus_sign = true;
                    chars.push(NumericConstantChar::PlusSign);
                }
                '.' => {
                    if has_decimal_point {
                        return Err(NumericConstantConstructionError::HasDecimalPoint);
                    }

                    has_decimal_point = true;
                    chars.push(NumericConstantChar::DecimalPoint);
                }
                'i' => {
                    if has_imaginary_unit {
                        return Err(NumericConstantConstructionError::HasImaginaryUnit);
                    }

                    has_imaginary_unit = true;
                    chars.push(NumericConstantChar::ImaginaryUnit);
                }
                _ => {}
            }
        }

        Ok(NumericConstant::new(chars, 10))
    }
}

impl Display for NumericConstant {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for current in &self.chars {
            write!(f, "{}", current)?;
        }

        Ok(())
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum NumericConstantConstructionError {
    HasMinusSign,
    HasPlusSign,
    HasDecimalPoint,
    HasImaginaryUnit
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum NumericConstantChar {
    Zero,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    A,
    B,
    C,
    D,
    E,
    F,
    MinusSign,
    PlusSign,
    DecimalPoint,
    ImaginaryUnit
}

impl Display for NumericConstantChar {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            NumericConstantChar::Zero => { write!(f, "0") }
            NumericConstantChar::One => { write!(f, "1") }
            NumericConstantChar::Two => { write!(f, "2") }
            NumericConstantChar::Three => { write!(f, "3") }
            NumericConstantChar::Four => { write!(f, "4") }
            NumericConstantChar::Five => { write!(f, "5") }
            NumericConstantChar::Six => { write!(f, "6") }
            NumericConstantChar::Seven => { write!(f, "7") }
            NumericConstantChar::Eight => { write!(f, "8") }
            NumericConstantChar::Nine => { write!(f, "9") }
            NumericConstantChar::A => { write!(f, "a") }
            NumericConstantChar::B => { write!(f, "b") }
            NumericConstantChar::C => { write!(f, "c") }
            NumericConstantChar::D => { write!(f, "d") }
            NumericConstantChar::E => { write!(f, "e") }
            NumericConstantChar::F => { write!(f, "f") }
            NumericConstantChar::MinusSign => { write!(f, "-") }
            NumericConstantChar::PlusSign => { write!(f, "+") }
            NumericConstantChar::DecimalPoint => { write!(f, ".") }
            NumericConstantChar::ImaginaryUnit => { write!(f, "i") }
        }
    }
}

#[test]
fn test_construct1() {
    assert_eq!(
        Ok(
            NumericConstant::new(
                vec![NumericConstantChar::Three, NumericConstantChar::One, NumericConstantChar::Three],
                10
            )
        ),
        NumericConstant::from_str("313")
    )
}

#[test]
fn test_construct2() {
    assert_eq!(
        Ok(
            NumericConstant::new(
                vec![
                    NumericConstantChar::Three, NumericConstantChar::One, NumericConstantChar::Three,
                    NumericConstantChar::DecimalPoint,
                    NumericConstantChar::Four, NumericConstantChar::Four
                ],
                10
            )
        ),
        NumericConstant::from_str("313.44")
    )
}

#[test]
fn test_construct3() {
    assert_eq!(
        Ok(
            NumericConstant::new(
                vec![
                    NumericConstantChar::One, NumericConstantChar::Two,
                    NumericConstantChar::PlusSign,
                    NumericConstantChar::Five, NumericConstantChar::ImaginaryUnit
                ],
                10
            )
        ),
        NumericConstant::from_str("12 + 5i")
    )
}

#[test]
fn test_construct4() {
    assert_eq!(
        NumericConstant::new(
            vec![NumericConstantChar::Four, NumericConstantChar::Two, NumericConstantChar::Four, NumericConstantChar::Five],
            10
        ),
        NumericConstant::from_i64(4245_i64)
    )
}

#[test]
fn test_construct5() {
    assert_eq!(
        NumericConstant::new(
            vec![NumericConstantChar::One, NumericConstantChar::Two, NumericConstantChar::DecimalPoint, NumericConstantChar::Zero],
            10
        ),
        NumericConstant::from_f64(12.0)
    )
}

#[test]
fn test_construct6() {
    assert_eq!(
        NumericConstant::new(
            vec![
                NumericConstantChar::One, NumericConstantChar::Three,
                NumericConstantChar::DecimalPoint,
                NumericConstantChar::Three,
                NumericConstantChar::Six, NumericConstantChar::Nine, NumericConstantChar::Nine, NumericConstantChar::Nine, NumericConstantChar::Nine, NumericConstantChar::Nine,
                NumericConstantChar::Nine, NumericConstantChar::Nine, NumericConstantChar::Nine, NumericConstantChar::Nine,
                NumericConstantChar::Nine, NumericConstantChar::Nine, NumericConstantChar::Nine, NumericConstantChar::Nine, NumericConstantChar::Two, NumericConstantChar::Two
            ],
            10
        ),
        NumericConstant::from_f64(13.37)
    )
}

#[test]
fn test_to_string1() {
    assert_eq!(
        "4711",
        NumericConstant::from_str("4711").unwrap().to_string()
    )
}

#[test]
fn test_to_string2() {
    assert_eq!(
        "13.37",
        NumericConstant::from_str("13.37").unwrap().to_string()
    )
}

#[test]
fn test_to_string3() {
    assert_eq!(
        "12+5i",
        NumericConstant::from_str("12+5i").unwrap().to_string()
    )
}