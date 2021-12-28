use derive_more::Deref;
use internment::LocalIntern;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deref)]
pub struct ConstantString(LocalIntern<Vec<u8>>);

impl ConstantString {
    pub fn new(data: Vec<u8>) -> Self {
        Self(LocalIntern::new(data))
    }

    pub fn data(&self) -> &Vec<u8> {
        &*self.0
    }
}

impl From<&str> for ConstantString {
    fn from(s: &str) -> Self {
        Self::new(Vec::from(s.as_bytes()))
    }
}

impl PartialEq<&str> for ConstantString {
    fn eq(&self, other: &&str) -> bool {
        self.0.as_slice() == other.as_bytes()
    }
}
