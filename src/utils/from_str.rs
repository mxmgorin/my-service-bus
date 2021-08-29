pub trait FromStr {
    fn from_str(src: &str) -> Result<Self, String>
    where
        Self: Sized;
}
