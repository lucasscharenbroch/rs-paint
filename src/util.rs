pub trait Iterable {
    type Item;
    fn iter(&self) -> Box<dyn Iterator<Item = Self::Item>>;
}