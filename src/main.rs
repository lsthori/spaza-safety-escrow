// Update main.rs imports
use spaza_escrow::api::{SmsService, MobileCarrier};
use spaza_escrow::trust::TrustManager;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    // Initialize services
    let storage = Arc::new(MemoryStorage::new());
    let sms_service = SmsService::new(MobileCarrier::Safaricom, true); // Simulated SMS
    let trust_manager = TrustManager::new(Arc::clone(&storage));
    
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Create(args) => handle_create(&storage, &sms_service, &trust_manager, args).await,
        Commands::Fund(args) => handle_fund(&storage, args),
        Commands::Release(args) => handle_release(&storage, &sms_service, args).await,
        Commands::Cancel(args) => handle_cancel(&storage, args),
        Commands::Dispute(args) => handle_dispute(&storage, &sms_service, args).await,
        Commands::Vote(args) => handle_vote(&storage, args),
        Commands::List => handle_list(&storage),
        Commands::Get(args) => handle_get(&storage, args),
        Commands::Trust(args) => handle_trust(&trust_manager, args),
        Commands::Demo(args) => handle_demo(&storage, &sms_service, &trust_manager, args).await,
        Commands::Dashboard => handle_dashboard(&storage, &trust_manager),
    }
}

async fn handle_create(
    storage: &Arc<MemoryStorage>,
    sms_service: &SmsService,
    trust_manager: &TrustManager,
    args: CreateArgs,
) -> Result<(), Box<dyn std::error::Error>> {
    // Get or create users
    let buyer = get_or_create_user(storage, args.buyer_id, &args.buyer_phone, "Buyer").await?;
    let seller = get_or_create_user(storage, args.seller_id, &args.seller_phone, "Seller").await?;
    
    // Calculate recommended duration based on trust
    let buyer_trust = trust_manager.get_score(args.buyer_id).await?;
    let seller_trust = trust_manager.get_score(args.seller_id).await?;
    let recommended_days = trust_manager.recommended_duration(buyer_trust, seller_trust);
    
    let amount = Decimal::from_f64(args.amount)
        .ok_or("Invalid amount")?;
    
    // Create escrow
    let escrow = Escrow::new(
        amount,
        args.currency,
        args.buyer_id,
        args.seller_id,
        args.description,
        recommended_days,
    );
    
    storage.create_escrow(escrow.clone())?;
    
    // Send SMS notifications
    if args.simulate_sms {
        if let Some(pin) = &escrow.release_pin {
            sms_service.send_pin_to_buyer(
                &args.buyer_phone,
                pin,
                &escrow.id,
                args.amount,
                &args.currency,
            ).await?;
            
            sms_service.notify_seller_delivery(
                &args.seller_phone,
                &escrow.id,
                args.amount,
                &args.currency,
            ).await?;
        }
    }
    
    // Print success with colors
    println!("\n{}", "✅ ESCROW CREATED SUCCESSFULLY!".green());
    println!("{}", "=".repeat(40).cyan());
    println!("{} {}", "🆔 ID:".bold(), escrow.id);
    println!("{} {} {}", "💰 Amount:".bold(), escrow.amount, escrow.currency);
    println!("{} {} days", "📅 Duration:".bold(), recommended_days);
    println!("{} {}", "👤 Buyer:".bold(), args.buyer_phone);
    println!("{} {}", "🏪 Seller:".bold(), args.seller_phone);
    if let Some(pin) = &escrow.release_pin {
        println!("{} {}", "🔐 PIN:".bold().red(), pin);
    }
    println!("{}", "=".repeat(40).cyan());
    
    Ok(())
}

async fn handle_demo(
    storage: &Arc<MemoryStorage>,
    sms_service: &SmsService,
    trust_manager: &TrustManager,
    args: DemoArgs,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n{}", "🚀 SPAZA SAFETY ESCROW DEMO".bold().cyan());
    println!("{}", "=".repeat(50).cyan());
    
    match args.scenario {
        1 => demo_scenario_1(storage, sms_service, trust_manager, args.with_sms).await,
        2 => demo_scenario_2(storage, sms_service, trust_manager, args.with_sms).await,
        _ => demo_scenario_1(storage, sms_service, trust_manager, args.with_sms).await,
    }
    
    Ok(())
}

async fn demo_scenario_1(
    storage: &Arc<MemoryStorage>,
    sms_service: &SmsService,
    trust_manager: &TrustManager,
    with_sms: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📋 Scenario 1: Successful Spaza Transaction");
    println!("{}", "-".repeat(40));
    println!("👨‍💼 Spaza Owner: Thabo (Trust: 75%)");
    println!("🏭 Wholesaler: Makro (Trust: 92%)");
    println!("💰 Amount: R1,500 for maize meal stock");
    println!("{}", "-".repeat(40));
    
    // Simulate the flow
    println!("\n1. {} Creating escrow...", "▶".green());
    // ... demo steps
    
    Ok(())
}