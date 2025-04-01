pub struct Required;

impl Required {
    pub fn apply<T>(value: &Option<T>) -> bool {
        value.is_some()
    }
}