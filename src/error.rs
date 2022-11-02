use stackable_operator::error::Error as StackableOperatorError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Stackable Operator Error: {0}")]
    StackableOperator(#[from] StackableOperatorError), // FIXME - we dont want this really

    #[error("No Before Create is configured")]
    NoBeforeCreate,
}
