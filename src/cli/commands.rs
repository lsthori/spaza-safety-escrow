use clap::{Args, Subcommand};
use uuid::Uuid;

#[derive(Subcommand)]
pub enum Commands {
    Create(CreateArgs),
    Fund(FundArgs),
    Release(ReleaseArgs),
    Cancel(CancelArgs),
    Dispute(DisputeArgs),
    Vote(VoteArgs),
    List,
    Get(GetArgs),
    Trust(TrustArgs),
    Demo(DemoArgs),
    Sms(SmsArgs),
    Dashboard,
}

#[derive(Args)]
pub struct CreateArgs {
    #[arg(short, long)]
    pub amount: f64,
    
    #[arg(short, long, default_value = "ZAR")]
    pub currency: String,
    
    #[arg(short = 'b', long)]
    pub buyer_id: Uuid,
    
    #[arg(long)]
    pub buyer_phone: String,
    
    #[arg(short = 's', long)]
    pub seller_id: String,
    
    #[arg(long)]
    pub seller_phone: String,
    
    #[arg(short, long, default_value = "Monthly stock purchase")]
    pub description: String,
    
    #[arg(short, long, default_value_t = 30)]
    pub days: i64,
    
    #[arg(long, default_value_t = false)]
    pub with_sms: bool,
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

#[derive(Args)]
pub struct TrustArgs {
    #[arg(short, long)]
    pub user_id: Uuid,
}

#[derive(Args)]
pub struct DemoArgs {
    #[arg(short, long, default_value_t = 1)]
    pub scenario: u8,
}

#[derive(Args)]
pub struct SmsArgs {
    #[arg(short, long)]
    pub phone: String,
    
    #[arg(short, long)]
    pub message: String,
}