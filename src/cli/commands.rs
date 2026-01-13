use clap::{Args, Subcommand};
use rust_decimal::Decimal;
use uuid::Uuid;

// Update CreateArgs to include phone numbers
#[derive(Args)]
pub struct CreateArgs {
    #[arg(short, long)]
    pub amount: f64,
    
    #[arg(short, long, default_value = "ZAR")]
    pub currency: String,
    
    #[arg(short = 'b', long)]
    pub buyer_id: Uuid,
    
    #[arg(short = 'B', long)]
    pub buyer_phone: String,
    
    #[arg(short = 's', long)]
    pub seller_id: Uuid,
    
    #[arg(short = 'S', long)]
    pub seller_phone: String,
    
    #[arg(short = 'd', long, default_value = "Monthly stock purchase")]
    pub description: String,
    
    #[arg(short = 't', long, default_value_t = 30)]
    pub days: i64,
    
    #[arg(long, default_value_t = false)]
    pub simulate_sms: bool,
}

// Add new commands
#[derive(Subcommand)]
pub enum Commands {
    /// Create a new escrow
    Create(CreateArgs),
    
    /// Fund an existing escrow
    Fund(FundArgs),
    
    /// Release funds to seller
    Release(ReleaseArgs),
    
    /// Cancel an escrow
    Cancel(CancelArgs),
    
    /// Raise a dispute
    Dispute(DisputeArgs),
    
    /// Vote on a dispute
    Vote(VoteArgs),
    
    /// List all escrows
    List,
    
    /// Get escrow details
    Get(GetArgs),
    
    /// Show trust score
    Trust(TrustArgs),
    
    /// Run demo scenario
    Demo(DemoArgs),
    
    /// Show dashboard
    Dashboard,
}

#[derive(Args)]
pub struct TrustArgs {
    #[arg(short, long)]
    pub user_id: Uuid,
}

#[derive(Args)]
pub struct DemoArgs {
    #[arg(short, long, default_value_t = 1)]
    pub scenario: u8,
    
    #[arg(long, default_value_t = false)]
    pub with_sms: bool,
}