use std::fmt::Debug;

#[derive(Debug, thiserror::Error)]
pub enum AsyncError<E>
where
    E: 'static + Debug + Send + Sync,
{
    // Error occured when attempting to deliver the query to the Database actor
    #[error("{}", 0)]
    Delivery(#[from] actix::MailboxError),

    // Timed out trying to checkout a connection
    #[error("{}", 0)]
    Timeout(#[from]  r2d2::Error),

    // An error occurred when interacting with the database
    #[error("{}", 0)]
    Execute(E),
}