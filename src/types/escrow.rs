use chrono::{DateTime, Utc, Duration};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EscrowState {
    Created,
    Funded,
    Completed,
    Cancelled,
    InDispute,
    Refunded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Escrow {
    pub id: Uuid,
    pub amount: Decimal,
    pub currency: String,
    pub buyer_id: Uuid,
    pub seller_id: Uuid,
    pub description: String,
    pub state: EscrowState,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub funded_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub release_pin: Option<String>,
    pub arbitrators: Vec<Uuid>,
    pub dispute_resolution: Option<DisputeResolution>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisputeResolution {
    pub raised_by: Uuid,
    pub raised_at: DateTime<Utc>,
    pub votes: Vec<Vote>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub decision: Option<DisputeDecision>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub arbitrator_id: Uuid,
    pub vote: bool,
    pub voted_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DisputeDecision {
    ReleaseToSeller,
    RefundToBuyer,
}

impl Escrow {
    pub fn new(
        amount: Decimal,
        currency: impl Into<String>,
        buyer_id: Uuid,
        seller_id: impl Into<String>,
        description: impl Into<String>,
        days_to_expire: i64,
    ) -> Self {
        let now = Utc::now();
        let release_pin = Self::generate_pin();
        
        Self {
            id: Uuid::new_v4(),
            amount,
            currency: currency.into(),
            buyer_id,
            seller_id: Uuid::parse_str(&seller_id.into()).unwrap_or_else(|_| Uuid::new_v4()),
            description: description.into(),
            state: EscrowState::Created,
            created_at: now,
            expires_at: now + Duration::days(days_to_expire),
            funded_at: None,
            completed_at: None,
            release_pin: Some(release_pin),
            arbitrators: vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()],
            dispute_resolution: None,
        }
    }
    
    fn generate_pin() -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        format!("{:06}", rng.gen_range(100000..999999))
    }
    
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
}