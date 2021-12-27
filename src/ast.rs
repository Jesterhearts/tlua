use bumpalo::Bump;
use tracing::instrument;

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
    #[instrument(level = "trace", name = "alloc", skip(self, val), fields(total_mem = self.allocated_bytes(), chunk_remain = self.0.chunk_capacity()))]
    pub(crate) fn alloc<T>(&self, val: T) -> &mut T {
        #[cfg(feature = "trace_mem")]
        let start_mem = self.allocated_bytes();

        #[allow(clippy::let_and_return)] // This binding is used in tracing.
        let v = self.0.alloc(val);

        #[cfg(feature = "trace_mem")]
        tracing::trace!(size = self.allocated_bytes() - start_mem, "allocated");

        v
    }
}

impl Default for ASTAllocator {
    fn default() -> Self {
        Self(Bump::new())
    }
}
