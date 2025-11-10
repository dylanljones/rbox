// Author: Dylan Jones
// Date:   2025-11-10

/// A trait for models that implement reordering via a sequence number.
pub trait Sequence {
    /// Returns the sequence number of the model
    ///
    /// The sequence number defines the order of the model in a list or collection.
    /// It is one-based (i.e., starts from 1).
    fn seq(&self) -> i32;
}
