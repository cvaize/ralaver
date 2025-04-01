pub struct Email;

impl Email {
    pub fn apply(value: &String) -> bool {
        value.len() <= 254 && value.contains("@")
    }
}
