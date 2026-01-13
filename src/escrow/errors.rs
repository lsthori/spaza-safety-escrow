use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum EscrowError {
    #[error("Escrow not found: {0}")]
    NotFound(Uuid),
    
    #[error("Invalid state transition from {from:?} to {to:?}")]
    InvalidStateTransition {
        from: String,
        to: String,
    },
    
    #[error("Insufficient funds. Required: {required}, Provided: {provided}")]
    InsufficientFunds {
        required: rust_decimal::Decimal,
        provided: rust_decimal::Decimal,
    },
    
    #[error("Unauthorized access by user: {0}")]
    Unauthorized(Uuid),
    
    #[error("Invalid release PIN")]
    InvalidPin,
    
    #[error("Escrow has expired")]
    Expired,
    
    #[error("Dispute already resolved")]
    DisputeAlreadyResolved,
    
    #[error("User is not an arbitrator")]
    NotArbitrator,
    
    #[error("Storage error: {0}")]
    StorageError(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
}