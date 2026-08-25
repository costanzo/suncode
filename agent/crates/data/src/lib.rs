mod domain;
mod operations;
mod schema;
mod store;

use diesel::result::Error as DieselError;

pub use domain::*;
pub use store::{ApprovalInput, Store};
pub use suncode_common::BusinessError;

pub(crate) fn database_error(error: DieselError) -> BusinessError {
    BusinessError::database(error.to_string())
}
