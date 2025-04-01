pub struct MinLengthString;
pub struct MaxLengthString;
pub struct MinMaxLengthString;

impl MinLengthString {
    pub fn apply(value: &String, min: usize) -> bool {
        value.len() >= min
    }
}

impl MaxLengthString {
    pub fn apply(value: &String, max: usize) -> bool {
        value.len() <= max
    }
}

impl MinMaxLengthString {
    pub fn apply(value: &String, min: usize, max: usize) -> bool {
        MinLengthString::apply(value, min) && MaxLengthString::apply(value, max)
    }
}
