use std::ops::Deref;

use internment::LocalIntern;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Ident(pub(crate) LocalIntern<Vec<u8>>);

impl Deref for Ident {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.0.as_slice()
    }
}

impl Ident {
    pub fn new_from_slice(data: &[u8]) -> Self {
        let mut vec = Vec::with_capacity(data.len());
        vec.extend_from_slice(data);
        Self(LocalIntern::new(vec))
    }
}

impl std::fmt::Debug for Ident {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Ident")
            .field(&String::from_utf8_lossy(&*self.0))
            .finish()
    }
}

impl<'chunk> ToString for Ident {
    fn to_string(&self) -> String {
        String::from_utf8_lossy(&*self.0).to_string()
    }
}

impl From<&str> for Ident {
    fn from(s: &str) -> Self {
        Self::new_from_slice(s.as_bytes())
    }
}
