use clap::{Args, Subcommand};
use rust_decimal::Decimal;
use uuid::Uuid;

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
}

#[derive(Args)]
pub struct CreateArgs {
    #[arg(short, long)]
    pub amount: f64,
    
    #[arg(short, long, default_value = "ZAR")]
    pub currency: String,
    
    #[arg(short = 'b', long)]
    pub buyer_id: Uuid,
    
    #[arg(short = 's', long)]
    pub seller_id: Uuid,
    
    #[arg(short, long, default_value = "Monthly stock purchase")]
    pub description: String,
    
    #[arg(short, long, default_value_t = 30)]
    pub days: i64,
}

#[derive(Args)]
pub struct FundArgs {
    #[arg(short, long)]
    pub escrow_id: Uuid,
    
    #[arg(short, long)]
    pub amount: f64,
}

#[derive(Args)]
pub struct ReleaseArgs {
    #[arg(short, long)]
    pub escrow_id: Uuid,
    
    #[arg(short, long)]
    pub user_id: Uuid,
    
    #[arg(short, long)]
    pub pin: String,
}

#[derive(Args)]
pub struct CancelArgs {
    #[arg(short, long)]
    pub escrow_id: Uuid,
    
    #[arg(short, long)]
    pub user_id: Uuid,
}

#[derive(Args)]
pub struct DisputeArgs {
    #[arg(short, long)]
    pub escrow_id: Uuid,
    
    #[arg(short, long)]
    pub user_id: Uuid,
}

#[derive(Args)]
pub struct VoteArgs {
    #[arg(short, long)]
    pub escrow_id: Uuid,
    
    #[arg(short, long)]
    pub arbitrator_id: Uuid,
    
    #[arg(short, long)]
    pub vote: bool,
}

#[derive(Args)]
pub struct GetArgs {
    #[arg(short, long)]
    pub escrow_id: Uuid,
}