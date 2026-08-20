// Domain specific errors for user-management
use thiserror::Error;

#[derive(Error, Debug)]
pub enum UserError {
    #[error("User not found")]
    NotFound,
}
