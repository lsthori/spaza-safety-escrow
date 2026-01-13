// src/trust/scoring.rs
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use chrono::{Utc, Duration};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustScore {
    pub user_id: Uuid,
    pub score: Decimal, // 0.0 to 100.0
    pub total_transactions: u32,
    pub successful_transactions: u32,
    pub disputed_transactions: u32,
    pub total_amount: Decimal,
    pub avg_transaction_time: i64, // seconds
    pub last_activity: chrono::DateTime<Utc>,
    pub created_at: chrono::DateTime<Utc>,
    pub penalties: Vec<Penalty>,
    pub bonuses: Vec<Bonus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrustLevel {
    Newbie,      // < 30
    Bronze,      // 30-59
    Silver,      // 60-79
    Gold,        // 80-89
    Platinum,    // 90-95
    Trusted,     // 96-100
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Penalty {
    pub reason: String,
    pub points: Decimal,
    pub timestamp: chrono::DateTime<Utc>,
    pub expires_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bonus {
    pub reason: String,
    pub points: Decimal,
    pub timestamp: chrono::DateTime<Utc>,
}

impl TrustScore {
    pub fn new(user_id: Uuid) -> Self {
        let now = Utc::now();
        Self {
            user_id,
            score: dec!(50.0), // Start at 50
            total_transactions: 0,
            successful_transactions: 0,
            disputed_transactions: 0,
            total_amount: dec!(0.0),
            avg_transaction_time: 0,
            last_activity: now,
            created_at: now,
            penalties: Vec::new(),
            bonuses: Vec::new(),
        }
    }

    pub fn update_after_transaction(
        &mut self,
        amount: Decimal,
        was_successful: bool,
        had_dispute: bool,
        transaction_time_seconds: i64,
    ) {
        self.total_transactions += 1;
        self.total_amount += amount;
        self.last_activity = Utc::now();
        
        if was_successful {
            self.successful_transactions += 1;
        }
        
        if had_dispute {
            self.disputed_transactions += 1;
        }
        
        // Update average transaction time (moving average)
        if self.total_transactions == 1 {
            self.avg_transaction_time = transaction_time_seconds;
        } else {
            self.avg_transaction_time = 
                (self.avg_transaction_time * (self.total_transactions - 1) as i64 + transaction_time_seconds) 
                / self.total_transactions as i64;
        }
        
        // Recalculate score
        self.score = calculate_trust_score(self);
    }

    pub fn add_penalty(&mut self, reason: String, points: Decimal, days_valid: i64) {
        let now = Utc::now();
        let penalty = Penalty {
            reason,
            points,
            timestamp: now,
            expires_at: now + Duration::days(days_valid),
        };
        
        self.penalties.push(penalty);
        self.score = self.score - points;
        self.score = self.score.max(dec!(0.0));
    }

    pub fn add_bonus(&mut self, reason: String, points: Decimal) {
        let bonus = Bonus {
            reason,
            points,
            timestamp: Utc::now(),
        };
        
        self.bonuses.push(bonus);
        self.score = self.score + points;
        self.score = self.score.min(dec!(100.0));
    }

    pub fn get_trust_level(&self) -> TrustLevel {
        match self.score {
            s if s < dec!(30.0) => TrustLevel::Newbie,
            s if s < dec!(60.0) => TrustLevel::Bronze,
            s if s < dec!(80.0) => TrustLevel::Silver,
            s if s < dec!(90.0) => TrustLevel::Gold,
            s if s < dec!(96.0) => TrustLevel::Platinum,
            _ => TrustLevel::Trusted,
        }
    }

    pub fn get_recommended_escrow_duration(&self) -> i64 {
        // Higher trust = shorter escrow duration
        match self.get_trust_level() {
            TrustLevel::Trusted => 1,    // 1 day
            TrustLevel::Platinum => 3,   // 3 days
            TrustLevel::Gold => 7,       // 1 week
            TrustLevel::Silver => 14,    // 2 weeks
            TrustLevel::Bronze => 30,    // 1 month
            TrustLevel::Newbie => 60,    // 2 months
        }
    }
}

/// Advanced trust scoring algorithm
pub fn calculate_trust_score(score: &TrustScore) -> Decimal {
    if score.total_transactions == 0 {
        return dec!(50.0);
    }

    let success_rate = Decimal::from(score.successful_transactions) 
        / Decimal::from(score.total_transactions) 
        * dec!(100.0);

    let dispute_rate = Decimal::from(score.disputed_transactions) 
        / Decimal::from(score.total_transactions) 
        * dec!(100.0);

    // Weight factors
    let success_weight = dec!(0.5);
    let volume_weight = dec!(0.2);
    let recency_weight = dec!(0.2);
    let speed_weight = dec!(0.1);
    let dispute_penalty = dec!(2.0); // Each dispute costs 2 points

    // Volume factor (logarithmic scale)
    let volume_factor = (score.total_amount.ln() / dec!(10.0).ln())
        .min(dec!(1.0)) * dec!(100.0);

    // Recency factor (active in last 30 days)
    let days_since_active = (Utc::now() - score.last_activity).num_days();
    let recency_factor = if days_since_active <= 30 {
        dec!(100.0) - (Decimal::from(days_since_active) * dec!(2.0))
    } else {
        dec!(40.0)
    };
    let recency_factor = recency_factor.max(dec!(0.0));

    // Speed factor (faster transactions = more trust)
    let speed_factor = if score.avg_transaction_time > 0 {
        let days_to_seconds = dec!(7.0 * 24.0 * 60.0 * 60.0); // 7 days in seconds
        let time_ratio = Decimal::from(score.avg_transaction_time) / days_to_seconds;
        (dec!(1.0) - time_ratio).max(dec!(0.0)) * dec!(100.0)
    } else {
        dec!(50.0)
    };

    // Calculate base score
    let base_score = 
        success_rate * success_weight +
        volume_factor * volume_weight +
        recency_factor * recency_weight +
        speed_factor * speed_weight;

    // Apply dispute penalty
    let penalty = dispute_rate * dispute_penalty;

    let mut final_score = base_score - penalty;

    // Apply active penalties
    let now = Utc::now();
    let active_penalties: Decimal = score.penalties
        .iter()
        .filter(|p| p.expires_at > now)
        .map(|p| p.points)
        .sum();
    
    final_score = final_score - active_penalties;

    // Add bonuses
    let bonuses: Decimal = score.bonuses
        .iter()
        .map(|b| b.points)
        .sum();
    
    final_score = final_score + bonuses;

    // Clamp between 0 and 100, round to 1 decimal
    final_score = final_score.max(dec!(0.0)).min(dec!(100.0));
    (final_score * dec!(10.0)).round() / dec!(10.0)
}