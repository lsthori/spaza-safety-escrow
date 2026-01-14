use clap::Parser;
use rust_decimal::prelude::*;
use rust_decimal::Decimal;
use spaza_safety_escrow::api::simulator::{MobileCarrier, SmsService};
use spaza_safety_escrow::cli::commands::{
    CancelArgs, Commands, CreateArgs, DemoArgs, DisputeArgs, FundArgs, GetArgs, ReleaseArgs,
    SmsArgs, TrustArgs, VoteArgs,
};
use spaza_safety_escrow::escrow::EscrowContract;
use spaza_safety_escrow::storage::memory::MemoryStorage;
use spaza_safety_escrow::trust::TrustManager;
use spaza_safety_escrow::types::Escrow;
use uuid::Uuid;

#[derive(Parser)]
#[command(name = "spaza-escrow")]
#[command(about = "Spaza Safety Escrow System", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let storage = MemoryStorage::new();
    let sms_service = SmsService::new(MobileCarrier::Safaricom);
    let mut trust_manager = TrustManager::new();

    let cli = Cli::parse();

    match cli.command {
        Commands::Create(args) => handle_create(&storage, &sms_service, &mut trust_manager, args),
        Commands::Fund(args) => handle_fund(&storage, args),
        Commands::Release(args) => handle_release(&storage, args),
        Commands::Cancel(args) => handle_cancel(&storage, args),
        Commands::Dispute(args) => handle_dispute(&storage, args),
        Commands::Vote(args) => handle_vote(&storage, args),
        Commands::List => handle_list(&storage),
        Commands::Get(args) => handle_get(&storage, args),
        Commands::Trust(args) => handle_trust(&trust_manager, args),
        Commands::Demo(args) => handle_demo(&storage, &sms_service, &mut trust_manager, args),
        Commands::Sms(args) => handle_sms(&sms_service, args),
        Commands::Dashboard => handle_dashboard(&storage, &trust_manager),
    }
}

fn handle_create(
    storage: &MemoryStorage,
    sms_service: &SmsService,
    trust_manager: &mut TrustManager,
    args: CreateArgs,
) -> Result<(), Box<dyn std::error::Error>> {
    let amount = Decimal::from_f64(args.amount).ok_or("Invalid amount")?;

    // Clone necessary values before moving
    let currency_clone = args.currency.clone();
    let seller_id_str = args.seller_id.clone();

    let escrow = Escrow::new(
        amount,
        args.currency,
        args.buyer_id,
        args.seller_id,
        args.description,
        args.days,
    );

    storage.create_escrow(escrow.clone())?;

    trust_manager.register_user(args.buyer_id);
    let seller_uuid = Uuid::parse_str(&seller_id_str)?;
    trust_manager.register_user(seller_uuid);

    if args.with_sms {
        if let Some(pin) = &escrow.release_pin {
            sms_service.send_pin_to_buyer(
                &args.buyer_phone,
                pin,
                &escrow.id.to_string(),
                args.amount,
                &currency_clone,
            )?;

            sms_service.notify_seller_delivery(
                &args.seller_phone,
                &escrow.id.to_string(),
                args.amount,
                &currency_clone,
            )?;
        }
    }

    println!("✅ Escrow created successfully!");
    println!("📋 ID: {}", escrow.id);
    println!("💰 Amount: {} {}", escrow.amount, currency_clone);
    println!("📅 Expires: {}", escrow.expires_at);
    println!("🔐 Release PIN: {}", escrow.release_pin.unwrap());
    println!(
        "📱 SMS notifications: {}",
        if args.with_sms { "SENT" } else { "NOT SENT" }
    );

    Ok(())
}

fn handle_fund(storage: &MemoryStorage, args: FundArgs) -> Result<(), Box<dyn std::error::Error>> {
    let mut escrow = storage
        .get_escrow(args.escrow_id)?
        .ok_or("Escrow not found")?;

    let amount = Decimal::from_f64(args.amount).ok_or("Invalid amount")?;

    EscrowContract::fund_escrow(&mut escrow, amount)?;
    storage.update_escrow(escrow)?;

    println!("✅ Escrow funded successfully!");
    Ok(())
}

fn handle_release(
    storage: &MemoryStorage,
    args: ReleaseArgs,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut escrow = storage
        .get_escrow(args.escrow_id)?
        .ok_or("Escrow not found")?;

    EscrowContract::release_to_seller(&mut escrow, args.user_id, &args.pin)?;
    storage.update_escrow(escrow)?;

    println!("✅ Funds released to seller!");
    Ok(())
}

fn handle_cancel(
    storage: &MemoryStorage,
    args: CancelArgs,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut escrow = storage
        .get_escrow(args.escrow_id)?
        .ok_or("Escrow not found")?;

    EscrowContract::cancel_escrow(&mut escrow, args.user_id)?;
    storage.update_escrow(escrow)?;

    println!("✅ Escrow cancelled!");
    Ok(())
}

fn handle_dispute(
    storage: &MemoryStorage,
    args: DisputeArgs,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut escrow = storage
        .get_escrow(args.escrow_id)?
        .ok_or("Escrow not found")?;

    EscrowContract::raise_dispute(&mut escrow, args.user_id)?;
    storage.update_escrow(escrow)?;

    println!("⚠️  Dispute raised! Waiting for arbitrator votes.");
    Ok(())
}

fn handle_vote(storage: &MemoryStorage, args: VoteArgs) -> Result<(), Box<dyn std::error::Error>> {
    let mut escrow = storage
        .get_escrow(args.escrow_id)?
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
    let escrow = storage
        .get_escrow(args.escrow_id)?
        .ok_or("Escrow not found")?;

    println!("{:#?}", escrow);
    Ok(())
}

fn handle_trust(
    trust_manager: &TrustManager,
    args: TrustArgs,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(profile) = trust_manager.get_profile(args.user_id) {
        println!("Trust Profile for User {}:", args.user_id);
        println!("  Score: {:.1}/100", profile.trust_score);
        println!("  Transactions: {}", profile.total_transactions);
        println!("  Successful: {}", profile.successful_transactions);
        println!("  Disputed: {}", profile.disputed_transactions);
        println!("  Total Amount: {}", profile.total_amount_transacted);
        println!("  Level: {:?}", profile.get_trust_level());
    } else {
        println!("User {} not found in trust system.", args.user_id);
    }
    Ok(())
}

fn handle_sms(sms_service: &SmsService, args: SmsArgs) -> Result<(), Box<dyn std::error::Error>> {
    sms_service.send(&args.phone, &args.message)?;
    println!("📱 SMS sent successfully!");
    Ok(())
}

fn handle_demo(
    storage: &MemoryStorage,
    sms_service: &SmsService,
    trust_manager: &mut TrustManager,
    _args: DemoArgs,
) -> Result<(), Box<dyn std::error::Error>> {
    use colored::*;

    println!("\n{}", "🚀 SPAZA SAFETY ESCROW DEMO".bold().cyan());
    let line = "=".repeat(50);
    println!("{}", line.cyan());

    let buyer_id = Uuid::new_v4();
    let seller_id = Uuid::new_v4();

    println!("\n📋 Scenario: Spaza shop buying stock from wholesaler");
    println!("👨‍💼 Buyer (Spaza Owner): {}", buyer_id);
    println!("🏭 Seller (Wholesaler): {}", seller_id);
    println!("💰 Amount: R1,500.00");
    let line2 = "=".repeat(50);
    println!("{}", line2.cyan());

    println!("\n1️⃣ Creating escrow...");
    let amount = Decimal::new(150000, 2);
    let escrow = Escrow::new(
        amount,
        "ZAR".to_string(),
        buyer_id,
        seller_id.to_string(),
        "Monthly stock purchase: maize, bread, milk".to_string(),
        7,
    );

    storage.create_escrow(escrow.clone())?;
    println!("   ✅ Escrow created: {}", escrow.id);

    trust_manager.register_user(buyer_id);
    trust_manager.register_user(seller_id);

    println!("\n2️⃣ Sending SMS notifications...");
    let pin_clone = escrow.release_pin.clone();
    if let Some(pin) = &pin_clone {
        sms_service.send_pin_to_buyer(
            "+27123456789",
            pin,
            &escrow.id.to_string(),
            1500.0,
            "ZAR",
        )?;
        sms_service.notify_seller_delivery(
            "+27876543210",
            &escrow.id.to_string(),
            1500.0,
            "ZAR",
        )?;
    }

    println!("\n3️⃣ Funding escrow...");
    let mut escrow = storage.get_escrow(escrow.id)?.unwrap();
    EscrowContract::fund_escrow(&mut escrow, amount)?;
    storage.update_escrow(escrow.clone())?;
    println!("   ✅ Escrow funded");

    println!("\n4️⃣ Updating trust scores...");
    trust_manager.record_transaction(buyer_id, amount, true, false)?;
    trust_manager.record_transaction(seller_id, amount, true, false)?;

    let buyer_trust = trust_manager.get_profile(buyer_id).unwrap();
    let seller_trust = trust_manager.get_profile(seller_id).unwrap();

    println!("   Buyer trust: {:.1}/100", buyer_trust.trust_score);
    println!("   Seller trust: {:.1}/100", seller_trust.trust_score);

    println!("\n5️⃣ Releasing funds with PIN...");
    let mut escrow = storage.get_escrow(escrow.id)?.unwrap();
    if let Some(pin) = pin_clone {
        EscrowContract::release_to_seller(&mut escrow, buyer_id, &pin)?;
        storage.update_escrow(escrow.clone())?;
        println!("   ✅ Funds released to seller");

        sms_service.notify_payment_released(
            "+27876543210",
            1500.0,
            "ZAR",
            &escrow.id.to_string(),
        )?;
    }

    println!("\n6️⃣ Final state:");
    println!("   Escrow State: {:?}", escrow.state);
    println!("   Completed at: {:?}", escrow.completed_at);

    println!("\n{}", "📊 FINAL DASHBOARD".bold().cyan());
    let line3 = "=".repeat(50);
    println!("{}", line3.cyan());
    println!("Total escrows: 1");
    println!("Active escrows: 0");
    println!("Total value: R1,500.00");
    println!(
        "Average trust score: {:.1}/100",
        (buyer_trust.trust_score + seller_trust.trust_score) / 2.0
    );
    let line4 = "=".repeat(50);
    println!("{}", line4.cyan());

    println!("\n✅ DEMO COMPLETED SUCCESSFULLY!");

    Ok(())
}

fn handle_dashboard(
    storage: &MemoryStorage,
    trust_manager: &TrustManager,
) -> Result<(), Box<dyn std::error::Error>> {
    use colored::*;

    let escrows = storage.list_escrows()?;
    let stats = trust_manager.export_stats();

    let active_escrows = escrows
        .iter()
        .filter(|e| {
            matches!(
                e.state,
                spaza_safety_escrow::types::EscrowState::Created
                    | spaza_safety_escrow::types::EscrowState::Funded
            )
        })
        .count();
    let total_value: f64 = escrows
        .iter()
        .map(|e| e.amount.to_string().parse::<f64>().unwrap_or(0.0))
        .sum();

    println!("\n{}", "📊 SPAZA ESCROW DASHBOARD".bold().cyan());
    let line = "=".repeat(40);
    println!("{}", line.cyan());
    println!("Total Users: {}", stats.total_users);
    println!("Total Escrows: {}", escrows.len());
    println!("Active Escrows: {}", active_escrows);
    println!("Total Value: R{:.2}", total_value);
    println!("Average Trust Score: {:.1}/100", stats.avg_trust_score);
    println!("High Trust Users: {}", stats.high_trust_users);
    println!("Total Transactions: {}", stats.total_transactions);
    let line2 = "=".repeat(40);
    println!("{}", line2.cyan());

    println!("\n📋 RECENT ESCROWS");
    for escrow in escrows.iter().take(5) {
        println!(
            "  {}: {} {} ({:?})",
            &escrow.id.to_string()[..8],
            escrow.amount,
            escrow.currency,
            escrow.state
        );
    }

    Ok(())
}
