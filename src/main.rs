use clap::Parser;
use rust_decimal::Decimal;
use spaza_escrow::cli::commands::{Commands, CreateArgs, FundArgs, ReleaseArgs, CancelArgs, DisputeArgs, VoteArgs, GetArgs};
use spaza_escrow::escrow::EscrowContract;
use spaza_escrow::storage::memory::MemoryStorage;
use spaza_escrow::types::Escrow;
use uuid::Uuid;

#[derive(Parser)]
#[command(name = "spaza-escrow")]
#[command(about = "Spaza Safety Escrow System", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    let storage = MemoryStorage::new();
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Create(args) => handle_create(&storage, args),
        Commands::Fund(args) => handle_fund(&storage, args),
        Commands::Release(args) => handle_release(&storage, args),
        Commands::Cancel(args) => handle_cancel(&storage, args),
        Commands::Dispute(args) => handle_dispute(&storage, args),
        Commands::Vote(args) => handle_vote(&storage, args),
        Commands::List => handle_list(&storage),
        Commands::Get(args) => handle_get(&storage, args),
    }
}

fn handle_create(storage: &MemoryStorage, args: CreateArgs) -> Result<(), Box<dyn std::error::Error>> {
    let amount = Decimal::from_f64(args.amount)
        .ok_or("Invalid amount")?;
    
    let escrow = Escrow::new(
        amount,
        args.currency,
        args.buyer_id,
        args.seller_id,
        args.description,
        args.days,
    );
    
    storage.create_escrow(escrow.clone())?;
    
    println!("✅ Escrow created successfully!");
    println!("📋 ID: {}", escrow.id);
    println!("💰 Amount: {} {}", escrow.amount, escrow.currency);
    println!("📅 Expires: {}", escrow.expires_at);
    println!("🔐 Release PIN: {}", escrow.release_pin.unwrap());
    
    Ok(())
}

fn handle_fund(storage: &MemoryStorage, args: FundArgs) -> Result<(), Box<dyn std::error::Error>> {
    let mut escrow = storage.get_escrow(args.escrow_id)?
        .ok_or("Escrow not found")?;
    
    let amount = Decimal::from_f64(args.amount)
        .ok_or("Invalid amount")?;
    
    EscrowContract::fund_escrow(&mut escrow, amount)?;
    storage.update_escrow(escrow)?;
    
    println!("✅ Escrow funded successfully!");
    Ok(())
}

fn handle_release(storage: &MemoryStorage, args: ReleaseArgs) -> Result<(), Box<dyn std::error::Error>> {
    let mut escrow = storage.get_escrow(args.escrow_id)?
        .ok_or("Escrow not found")?;
    
    EscrowContract::release_to_seller(&mut escrow, args.user_id, &args.pin)?;
    storage.update_escrow(escrow)?;
    
    println!("✅ Funds released to seller!");
    Ok(())
}

fn handle_cancel(storage: &MemoryStorage, args: CancelArgs) -> Result<(), Box<dyn std::error::Error>> {
    let mut escrow = storage.get_escrow(args.escrow_id)?
        .ok_or("Escrow not found")?;
    
    EscrowContract::cancel_escrow(&mut escrow, args.user_id)?;
    storage.update_escrow(escrow)?;
    
    println!("✅ Escrow cancelled!");
    Ok(())
}

fn handle_dispute(storage: &MemoryStorage, args: DisputeArgs) -> Result<(), Box<dyn std::error::Error>> {
    let mut escrow = storage.get_escrow(args.escrow_id)?
        .ok_or("Escrow not found")?;
    
    EscrowContract::raise_dispute(&mut escrow, args.user_id)?;
    storage.update_escrow(escrow)?;
    
    println!("⚠️  Dispute raised! Waiting for arbitrator votes.");
    Ok(())
}

fn handle_vote(storage: &MemoryStorage, args: VoteArgs) -> Result<(), Box<dyn std::error::Error>> {
    let mut escrow = storage.get_escrow(args.escrow_id)?
        .ok_or("Escrow not found")?;
    
    EscrowContract::vote_on_dispute(&mut escrow, args.arbitrator_id, args.vote)?;
    storage.update_escrow(escrow)?;
    
    println!("✅ Vote recorded!");
    Ok(())
}

fn handle_list(storage: &MemoryStorage) -> Result<(), Box<dyn std::error::Error>> {
    let escrows = storage.list_escrows()?;
    
    println!("📋 Total escrows: {}", escrows.len());
    for escrow in escrows {
        println!("\n---");
        println!("ID: {}", escrow.id);
        println!("Amount: {} {}", escrow.amount, escrow.currency);
        println!("State: {:?}", escrow.state);
        println!("Buyer: {}", escrow.buyer_id);
        println!("Seller: {}", escrow.seller_id);
    }
    
    Ok(())
}

fn handle_get(storage: &MemoryStorage, args: GetArgs) -> Result<(), Box<dyn std::error::Error>> {
    let escrow = storage.get_escrow(args.escrow_id)?
        .ok_or("Escrow not found")?;
    
    println!("{:#?}", escrow);
    Ok(())
}