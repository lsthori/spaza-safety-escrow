// src/api/simulator.rs
use std::fs::OpenOptions;
use std::io::Write;
use chrono::Utc;
use thiserror::Error;
use uuid::Uuid;
use colored::*;

#[derive(Error, Debug)]
pub enum SmsError {
    #[error("Failed to send SMS: {0}")]
    SendFailed(String),
    #[error("Phone number invalid: {0}")]
    InvalidPhone(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

#[derive(Debug, Clone)]
pub enum MobileCarrier {
    MTN,
    Vodacom,
    Airtel,
    Safaricom,
    Orange,
}

pub struct SmsService {
    carrier: MobileCarrier,
    sender_id: String,
    is_simulated: bool,
}

impl SmsService {
    pub fn new(carrier: MobileCarrier, is_simulated: bool) -> Self {
        Self {
            carrier,
            sender_id: "SPAZAESCROW".to_string(),
            is_simulated,
        }
    }

    /// Send a one-time PIN to buyer
    pub fn send_pin_to_buyer(&self, phone: &str, pin: &str, escrow_id: &Uuid, amount: f64, currency: &str) -> Result<String, SmsError> {
        let message = format!(
            "Spaza Escrow PIN: {}\nFor Escrow: {}\nAmount: {} {}\n\nGive this PIN to the delivery driver to release payment.\n\nReply HELP for support.",
            pin,
            &escrow_id.to_string()[..8],
            amount,
            currency
        );
        
        self.send(phone, &message)
    }

    /// Notify seller about pending delivery
    pub fn notify_seller_delivery(&self, phone: &str, escrow_id: &Uuid, amount: f64, currency: &str) -> Result<String, SmsError> {
        let message = format!(
            "✅ FUNDS GUARANTEED!\nEscrow: {}\nAmount: {} {}\n\nBuyer has escrowed funds. You can safely deliver goods.\n\nReply DELIVERED when done.",
            &escrow_id.to_string()[..8],
            amount,
            currency
        );
        
        self.send(phone, &message)
    }

    /// Notify about successful payment release
    pub fn notify_payment_released(&self, phone: &str, amount: f64, currency: &str, escrow_id: &Uuid) -> Result<String, SmsError> {
        let message = format!(
            "💰 PAYMENT RECEIVED!\nAmount: {} {}\nEscrow: {}\n\nFunds have been released to your account.\n\nThank you for using Spaza Safety!",
            amount,
            currency,
            &escrow_id.to_string()[..8]
        );
        
        self.send(phone, &message)
    }

    /// Send dispute notification to arbitrators
    pub fn notify_dispute(&self, phone: &str, escrow_id: &Uuid, amount: f64, currency: &str) -> Result<String, SmsError> {
        let message = format!(
            "⚖️ DISPUTE ALERT\nEscrow: {}\nAmount: {} {}\n\nPlease review and vote on this dispute.\n\nReply VOTE to participate.",
            &escrow_id.to_string()[..8],
            amount,
            currency
        );
        
        self.send(phone, &message)
    }

    /// Core send method with simulation
    fn send(&self, phone: &str, message: &str) -> Result<String, SmsError> {
        if self.is_simulated {
            self.simulate_send(phone, message)
        } else {
            // In production, integrate with Africa's Talking, Twilio, etc.
            self.real_send(phone, message)
        }
    }

    /// Simulate SMS sending (for demo)
    fn simulate_send(&self, phone: &str, message: &str) -> Result<String, SmsError> {
        let carrier_name = format!("{:?}", self.carrier);
        let timestamp = Utc::now().format("%H:%M:%S");
        
        println!("\n{}", "=".repeat(50).cyan());
        println!("{} SMS via {}", "📱".green(), carrier_name.blue());
        println!("{} To: {}", "   →".yellow(), phone);
        println!("{} Time: {}", "   →".yellow(), timestamp);
        println!("{}\n{}", "   →".yellow(), message);
        println!("{}\n", "=".repeat(50).cyan());
        
        // Log to file for audit trail
        self.log_sms(phone, message)?;
        
        Ok(format!("sms_{}", Utc::now().timestamp()))
    }

    /// Real SMS sending (stub for production)
    fn real_send(&self, _phone: &str, _message: &str) -> Result<String, SmsError> {
        // Integration with actual SMS provider
        Err(SmsError::SendFailed("Real SMS not implemented in demo".to_string()))
    }

    /// Log SMS for demo purposes
    fn log_sms(&self, phone: &str, message: &str) -> Result<(), SmsError> {
        let log_entry = format!(
            "[{}] {} | To: {} | Message: {}\n",
            Utc::now().to_rfc3339(),
            self.carrier.to_string(),
            Self::mask_phone(phone),
            message.replace("\n", " ")
        );

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open("sms_audit.log")?;

        file.write_all(log_entry.as_bytes())?;
        Ok(())
    }

    /// Mask phone number for privacy
    fn mask_phone(phone: &str) -> String {
        if phone.len() > 4 {
            format!("******{}", &phone[phone.len()-4..])
        } else {
            "****".to_string()
        }
    }
}

impl std::fmt::Display for MobileCarrier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MobileCarrier::MTN => write!(f, "MTN"),
            MobileCarrier::Vodacom => write!(f, "Vodacom"),
            MobileCarrier::Airtel => write!(f, "Airtel"),
            MobileCarrier::Safaricom => write!(f, "Safaricom (M-Pesa)"),
            MobileCarrier::Orange => write!(f, "Orange Money"),
        }
    }
}