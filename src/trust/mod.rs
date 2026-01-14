use std::collections::HashMap;
use chrono::Utc;
use rust_decimal::Decimal;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserTrustProfile {
    pub user_id: Uuid,
    pub trust_score: f64,
    pub total_transactions: u32,
    pub successful_transactions: u32,
    pub disputed_transactions: u32,
    pub total_amount_transacted: Decimal,
    pub last_active: chrono::DateTime<Utc>,
    pub join_date: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TrustLevel {
    Newbie,
    Bronze,
    Silver,
    Gold,
    Platinum,
    Trusted,
}

#[derive(Debug, Serialize)]
pub struct TrustStats {
    pub total_users: usize,
    pub avg_trust_score: f64,
    pub high_trust_users: usize,
    pub total_transactions: usize,
}

pub struct TrustManager {
    profiles: HashMap<Uuid, UserTrustProfile>,
}

impl TrustManager {
    pub fn new() -> Self {
        Self {
            profiles: HashMap::new(),
        }
    }

    pub fn register_user(&mut self, user_id: Uuid) {
        let now = Utc::now();
        let profile = UserTrustProfile {
            user_id,
            trust_score: 50.0,
            total_transactions: 0,
            successful_transactions: 0,
            disputed_transactions: 0,
            total_amount_transacted: Decimal::from(0),
            last_active: now,
            join_date: now,
        };
        
        self.profiles.insert(user_id, profile);
    }

    pub fn record_transaction(
        &mut self,
        user_id: Uuid,
        amount: Decimal,
        was_successful: bool,
        had_dispute: bool,
    ) -> Result<f64, String> {
        let profile = self.profiles.get_mut(&user_id)
            .ok_or_else(|| format!("User {} not found", user_id))?;

        profile.total_transactions += 1;
        profile.total_amount_transacted += amount;
        profile.last_active = Utc::now();

        if was_successful {
            profile.successful_transactions += 1;
        }

        if had_dispute {
            profile.disputed_transactions += 1;
        }

        // Calculate new score
        let new_score = Self::calculate_trust_score(profile);
        profile.trust_score = new_score;

        Ok(new_score)
    }

    fn calculate_trust_score(profile: &UserTrustProfile) -> f64 {
        if profile.total_transactions == 0 {
            return 50.0;
        }

        let success_rate = (profile.successful_transactions as f64 / profile.total_transactions as f64) * 100.0;
        let dispute_rate = (profile.disputed_transactions as f64 / profile.total_transactions as f64) * 100.0;

        let base_score = success_rate * 0.7;
        let penalty = dispute_rate * 20.0;
        
        let mut final_score = base_score - penalty;
        final_score = final_score.max(0.0).min(100.0);
        
        (final_score * 10.0).round() / 10.0
    }

    pub fn get_profile(&self, user_id: Uuid) -> Option<&UserTrustProfile> {
        self.profiles.get(&user_id)
    }

    pub fn recommended_escrow_duration(&self, buyer_id: Uuid, seller_id: Uuid) -> i64 {
        let buyer_score = self.profiles.get(&buyer_id)
            .map(|p| p.trust_score)
            .unwrap_or(50.0);
        let seller_score = self.profiles.get(&seller_id)
            .map(|p| p.trust_score)
            .unwrap_or(50.0);

        let avg_score = (buyer_score + seller_score) / 2.0;

        match avg_score {
            s if s >= 90.0 => 1,
            s if s >= 70.0 => 3,
            s if s >= 50.0 => 7,
            _ => 14,
        }
    }

    pub fn export_stats(&self) -> TrustStats {
        let total_users = self.profiles.len();
        let avg_score = if total_users > 0 {
            self.profiles.values()
                .map(|p| p.trust_score)
                .sum::<f64>() / total_users as f64
        } else {
            0.0
        };

        let high_trust = self.profiles.values()
            .filter(|p| p.trust_score >= 80.0)
            .count();

        TrustStats {
            total_users,
            avg_trust_score: avg_score,
            high_trust_users: high_trust,
            total_transactions: self.profiles.values()
                .map(|p| p.total_transactions as usize)
                .sum(),
        }
    }
}

impl UserTrustProfile {
    pub fn get_trust_level(&self) -> TrustLevel {
        match self.trust_score {
            s if s < 30.0 => TrustLevel::Newbie,
            s if s < 60.0 => TrustLevel::Bronze,
            s if s < 80.0 => TrustLevel::Silver,
            s if s < 90.0 => TrustLevel::Gold,
            s if s < 96.0 => TrustLevel::Platinum,
            _ => TrustLevel::Trusted,
        }
    }
}