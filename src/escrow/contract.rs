use crate::types::escrow::{Escrow, EscrowState, DisputeResolution, Vote, DisputeDecision};
use crate::escrow::errors::EscrowError;
use chrono::Utc;
use rust_decimal::Decimal;
use uuid::Uuid;

pub struct EscrowContract;

impl EscrowContract {
    pub fn fund_escrow(escrow: &mut Escrow, amount: Decimal) -> Result<(), EscrowError> {
        if escrow.state != EscrowState::Created {
            return Err(EscrowError::InvalidStateTransition {
                from: format!("{:?}", escrow.state),
                to: "Funded".to_string(),
            });
        }
        
        if amount < escrow.amount {
            return Err(EscrowError::InsufficientFunds {
                required: escrow.amount,
                provided: amount,
            });
        }
        
        escrow.state = EscrowState::Funded;
        escrow.funded_at = Some(Utc::now());
        Ok(())
    }
    
    pub fn release_to_seller(escrow: &mut Escrow, user_id: Uuid, pin: &str) -> Result<(), EscrowError> {
        if escrow.state != EscrowState::Funded {
            return Err(EscrowError::InvalidStateTransition {
                from: format!("{:?}", escrow.state),
                to: "Completed".to_string(),
            });
        }
        
        if escrow.is_expired() {
            return Err(EscrowError::Expired);
        }
        
        if escrow.buyer_id != user_id {
            return Err(EscrowError::Unauthorized(user_id));
        }
        
        if escrow.release_pin.as_deref() != Some(pin) {
            return Err(EscrowError::InvalidPin);
        }
        
        escrow.state = EscrowState::Completed;
        escrow.completed_at = Some(Utc::now());
        escrow.release_pin = None; // Invalidate PIN after use
        
        Ok(())
    }
    
    pub fn raise_dispute(escrow: &mut Escrow, user_id: Uuid) -> Result<(), EscrowError> {
        if escrow.state != EscrowState::Funded {
            return Err(EscrowError::InvalidStateTransition {
                from: format!("{:?}", escrow.state),
                to: "InDispute".to_string(),
            });
        }
        
        if escrow.buyer_id != user_id && escrow.seller_id != user_id {
            return Err(EscrowError::Unauthorized(user_id));
        }
        
        escrow.state = EscrowState::InDispute;
        escrow.dispute_resolution = Some(DisputeResolution {
            raised_by: user_id,
            raised_at: Utc::now(),
            votes: Vec::new(),
            resolved_at: None,
            decision: None,
        });
        
        Ok(())
    }
    
    pub fn vote_on_dispute(
        escrow: &mut Escrow, 
        arbitrator_id: Uuid, 
        vote: bool
    ) -> Result<(), EscrowError> {
        if escrow.state != EscrowState::InDispute {
            return Err(EscrowError::InvalidStateTransition {
                from: format!("{:?}", escrow.state),
                to: "InDispute".to_string(),
            });
        }
        
        // Check if user is an arbitrator for this escrow
        if !escrow.arbitrators.contains(&arbitrator_id) {
            return Err(EscrowError::NotArbitrator);
        }
        
        let dispute = escrow.dispute_resolution.as_mut()
            .ok_or_else(|| EscrowError::ValidationError("No dispute found".to_string()))?;
        
        // Check if already voted
        if dispute.votes.iter().any(|v| v.arbitrator_id == arbitrator_id) {
            return Err(EscrowError::ValidationError("Already voted".to_string()));
        }
        
        dispute.votes.push(Vote {
            arbitrator_id,
            vote,
            voted_at: Utc::now(),
        });
        
        // Check if we have majority (simple 2/3 majority)
        let total_arbitrators = escrow.arbitrators.len();
        let votes_for_release = dispute.votes.iter().filter(|v| v.vote).count();
        let votes_for_refund = dispute.votes.iter().filter(|v| !v.vote).count();
        
        // If majority reached, resolve dispute
        let needed_for_majority = (total_arbitrators * 2 + 2) / 3; // Ceiling of 2/3
        
        if votes_for_release >= needed_for_majority {
            dispute.decision = Some(DisputeDecision::ReleaseToSeller);
            dispute.resolved_at = Some(Utc::now());
            escrow.state = EscrowState::Completed;
            escrow.completed_at = Some(Utc::now());
        } else if votes_for_refund >= needed_for_majority {
            dispute.decision = Some(DisputeDecision::RefundToBuyer);
            dispute.resolved_at = Some(Utc::now());
            escrow.state = EscrowState::Refunded;
        }
        
        Ok(())
    }
    
    pub fn cancel_escrow(escrow: &mut Escrow, user_id: Uuid) -> Result<(), EscrowError> {
        if escrow.state != EscrowState::Created {
            return Err(EscrowError::InvalidStateTransition {
                from: format!("{:?}", escrow.state),
                to: "Cancelled".to_string(),
            });
        }
        
        if escrow.buyer_id != user_id {
            return Err(EscrowError::Unauthorized(user_id));
        }
        
        escrow.state = EscrowState::Cancelled;
        Ok(())
    }
    
    pub fn auto_refund_if_expired(escrow: &mut Escrow) -> Result<bool, EscrowError> {
        if escrow.state == EscrowState::Funded && escrow.is_expired() {
            escrow.state = EscrowState::Refunded;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}