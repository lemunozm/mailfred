use crate::router::Filter;

pub struct Any;

impl Filter for Any {
    fn check(&self, _: &str) -> bool {
        true
    }
}

pub struct StartWith(pub &'static str);

impl Filter for StartWith {
    fn check(&self, header: &str) -> bool {
        header.starts_with(self.0)
    }
}
