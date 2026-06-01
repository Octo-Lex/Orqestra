pub mod types;
pub mod error;
pub mod store;

pub use types::{Triple, CommitStub};
pub use error::GraphStoreError;
pub use store::{TripleStore, index_commits};
