use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub phone_number: String,
    pub user_type: UserType,
    pub trust_score: TrustScore,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserType {
    Buyer,
    Seller,
    Arbitrator,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustScore {
    pub score: Decimal,
    pub total_transactions: u32,
    pub successful_transactions: u32,
    pub dispute_rate: Decimal,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

impl TrustScore {
    pub fn new() -> Self {
        Self {
            score: Decimal::new(50, 0),
            total_transactions: 0,
            successful_transactions: 0,
            dispute_rate: Decimal::ZERO,
            last_updated: chrono::Utc::now(),
        }
    }
    
    pub fn update_after_success(&mut self) {
        self.total_transactions += 1;
        self.successful_transactions += 1;
        self.recalculate();
    }
    
    pub fn update_after_dispute(&mut self, won_dispute: bool) {
        self.total_transactions += 1;
        if won_dispute {
            self.successful_transactions += 1;
        }
        self.recalculate();
    }
    
    fn recalculate(&mut self) {
        if self.total_transactions > 0 {
            let success_rate = Decimal::from(self.successful_transactions) 
                / Decimal::from(self.total_transactions);
            
            self.score = (success_rate * Decimal::from(90)) + Decimal::from(10);
            self.score = self.score.min(Decimal::from(100)).max(Decimal::ZERO);
            
            self.dispute_rate = Decimal::from(self.total_transactions - self.successful_transactions)
                / Decimal::from(self.total_transactions);
        }
        self.last_updated = chrono::Utc::now();
    }
}