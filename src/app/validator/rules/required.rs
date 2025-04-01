pub struct Required;

#[allow(dead_code)]
impl Required {
    pub fn apply<T>(value: &Option<T>) -> bool {
        value.is_some()
    }
}