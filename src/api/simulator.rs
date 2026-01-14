use std::fs::OpenOptions;
use std::io::Write;
use chrono::Local;
use colored::*;

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
    log_file: String,
}

impl SmsService {
    pub fn new(carrier: MobileCarrier) -> Self {
        Self {
            carrier,
            log_file: "sms_log.txt".to_string(),
        }
    }

    pub fn send_pin_to_buyer(&self, phone: &str, pin: &str, escrow_id: &str, amount: f64, currency: &str) -> Result<(), std::io::Error> {
        let message = format!(
            "Spaza Escrow PIN: {}\nFor Escrow: {}\nAmount: {} {}\n\nGive this PIN to delivery driver to release payment.",
            pin, &escrow_id[..8], amount, currency
        );
        
        self.send(phone, &message)
    }

    pub fn notify_seller_delivery(&self, phone: &str, escrow_id: &str, amount: f64, currency: &str) -> Result<(), std::io::Error> {
        let message = format!(
            "FUNDS GUARANTEED!\nEscrow: {}\nAmount: {} {}\n\nBuyer has escrowed funds. You can safely deliver goods.",
            &escrow_id[..8], amount, currency
        );
        
        self.send(phone, &message)
    }

    pub fn notify_payment_released(&self, phone: &str, amount: f64, currency: &str, escrow_id: &str) -> Result<(), std::io::Error> {
        let message = format!(
            "PAYMENT RECEIVED!\nAmount: {} {}\nEscrow: {}\n\nFunds released to your account.",
            amount, currency, &escrow_id[..8]
        );
        
        self.send(phone, &message)
    }

    pub fn send(&self, phone: &str, message: &str) -> Result<(), std::io::Error> {
        let now = Local::now();
        let timestamp = now.format("%H:%M:%S");
        let carrier_name = format!("{:?}", self.carrier);
        
        println!("\n{}", "=".repeat(50).cyan());
        println!("{} SMS via {}", "📱".green(), carrier_name.blue());
        println!("{} To: {}", "→".yellow(), phone);
        println!("{} Time: {}", "→".yellow(), timestamp);
        println!("{}\n{}", "→".yellow(), message);
        println!("{}", "=".repeat(50).cyan());
        
        let log_entry = format!(
            "[{}] {} | To: {} | Message: {}\n",
            now.format("%Y-%m-%d %H:%M:%S"),
            carrier_name,
            Self::mask_phone(phone),
            message.replace("\n", " ")
        );

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_file)?;

        file.write_all(log_entry.as_bytes())?;
        
        Ok(())
    }

    fn mask_phone(phone: &str) -> String {
        if phone.len() > 4 {
            format!("******{}", &phone[phone.len()-4..])
        } else {
            "****".to_string()
        }
    }
}