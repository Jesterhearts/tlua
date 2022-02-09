use bumpalo::Bump;

pub mod block;
pub mod constant_string;
pub mod expressions;
pub mod identifiers;
pub mod prefix_expression;
pub mod statement;

#[derive(Debug)]
pub struct ASTAllocator(Bump);

impl ASTAllocator {
    pub fn allocated_bytes(&self) -> usize {
        self.0.allocated_bytes()
    }

    #[allow(clippy::mut_from_ref)] // I think bumpalo knows what it's doing
    pub fn alloc<T>(&self, val: T) -> &mut T {
        self.0.alloc(val)
    }
}

impl Default for ASTAllocator {
    fn default() -> Self {
        Self(Bump::new())
    }
}
