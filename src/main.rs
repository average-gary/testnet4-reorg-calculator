use anyhow::{Context, Result};
use bitcoincore_rpc::{Auth, Client, RpcApi};
use chrono::{DateTime, Utc};
use clap::Parser;
use dotenvy::dotenv;
use std::env;
use std::fs::OpenOptions;
use std::io::Write;

const HASHES_PER_DIFFICULTY: f64 = 4294967296.0; // 2^32
const SECONDS_PER_DAY: f64 = 86400.0;

#[derive(Parser, Debug)]
#[command(author, version, about = "Calculate Testnet4 reorg work requirements", long_about = None)]
struct Args {
    /// Fork block height to start reorg from
    #[arg(short, long)]
    fork_height: Option<u64>,
    
    /// Target completion time in days
    #[arg(short, long)]
    target_days: Option<f64>,
    
    /// Available hashrate in hashes/second
    #[arg(long)]
    hashrate: Option<f64>,
    
    /// RPC username
    #[arg(long)]
    rpcuser: Option<String>,
    
    /// RPC password
    #[arg(long)]
    rpcpassword: Option<String>,
    
    /// RPC port
    #[arg(long)]
    rpcport: Option<u16>,
    
    /// Calculate multiple target heights
    #[arg(long)]
    batch_calculate: bool,
}

#[derive(Debug, Clone)]
struct ReorgCalculation {
    fork_height: u64,
    current_height: u64,
    blocks_to_reorg: u64,
    total_work: f64,
    current_difficulty: f64,
    blocks_needed: f64,
    time_required_hours: f64,
    time_required_days: f64,
    hashrate_required: f64,
    timestamp: DateTime<Utc>,
}

fn load_config() -> Result<(String, String, String, u16, f64, f64)> {
    dotenv().ok();
    
    let rpc_url = env::var("RPC_URL").unwrap_or_else(|_| "http://127.0.0.1:48337".to_string());
    let rpc_user = env::var("RPC_USER").unwrap_or_else(|_| "myusername".to_string());
    let rpc_password = env::var("RPC_PASSWORD").unwrap_or_else(|_| "mypassword".to_string());
    let rpc_port = env::var("RPC_PORT")
        .unwrap_or_else(|_| "48337".to_string())
        .parse()
        .context("Invalid RPC_PORT in .env")?;
    let default_hashrate = env::var("DEFAULT_HASHRATE")
        .unwrap_or_else(|_| "1000000000000000".to_string())
        .parse()
        .context("Invalid DEFAULT_HASHRATE in .env")?;
    let target_days = env::var("TARGET_DAYS")
        .unwrap_or_else(|_| "3".to_string())
        .parse()
        .context("Invalid TARGET_DAYS in .env")?;
    
    Ok((rpc_url, rpc_user, rpc_password, rpc_port, default_hashrate, target_days))
}

fn connect_to_node(rpc_url: &str, rpc_user: &str, rpc_password: &str) -> Result<Client> {
    let client = Client::new(
        rpc_url,
        Auth::UserPass(rpc_user.to_string(), rpc_password.to_string()),
    )
    .context("Failed to create RPC client")?;
    
    // Test connection with a simple call that doesn't require network detection
    match client.get_block_count() {
        Ok(_) => Ok(client),
        Err(e) => Err(anyhow::anyhow!("Failed to connect to Bitcoin node: {}", e))
    }
}

fn get_block_difficulty(client: &Client, block_height: u64) -> Result<f64> {
    let block_hash = client.get_block_hash(block_height)
        .context(format!("Failed to get block hash for height {}", block_height))?;
    let block = client.get_block(&block_hash)
        .context(format!("Failed to get block for height {}", block_height))?;
    // Use bits to calculate difficulty directly
    let bits = block.header.bits.to_consensus();
    let difficulty = bits_to_difficulty(bits);
    Ok(difficulty)
}

fn bits_to_difficulty(bits: u32) -> f64 {
    let max_target = 0x1d00ffff_u32;
    let current_target = bits;
    
    // Convert bits to target
    let (current_mantissa, current_exponent) = ((current_target & 0xffffff) as f64, ((current_target >> 24) & 0xff) as i32);
    let (max_mantissa, max_exponent) = ((max_target & 0xffffff) as f64, ((max_target >> 24) & 0xff) as i32);
    
    let current_target_value = current_mantissa * 256_f64.powi(current_exponent - 3);
    let max_target_value = max_mantissa * 256_f64.powi(max_exponent - 3);
    
    max_target_value / current_target_value
}

fn calculate_chain_work(client: &Client, fork_height: u64, current_height: u64) -> Result<f64> {
    let mut total_work = 0.0;
    println!("Calculating chain work from block {} to {}...", fork_height, current_height);
    
    for height in fork_height..=current_height {
        let difficulty = get_block_difficulty(client, height)?;
        total_work += difficulty;
        
        if height % 1000 == 0 || height == current_height {
            println!("  Processed block {} (difficulty: {:.2})", height, difficulty);
        }
    }
    
    Ok(total_work)
}

fn calculate_reorg_requirements(
    client: &Client,
    fork_height: u64,
    hashrate: f64,
    target_days: f64,
) -> Result<ReorgCalculation> {
    let current_height = client.get_block_count()
        .context("Failed to get current block height")?;
    
    if fork_height > current_height {
        return Err(anyhow::anyhow!(
            "Fork height {} exceeds current chain height {}",
            fork_height,
            current_height
        ));
    }
    
    let current_difficulty = client.get_difficulty()
        .context("Failed to get current difficulty")?;
    
    let total_work = calculate_chain_work(client, fork_height, current_height)?;
    let blocks_to_reorg = current_height - fork_height + 1;
    
    // Calculate blocks needed to exceed existing chain work
    let blocks_needed = (total_work / current_difficulty).ceil();
    
    // Calculate time required with given hashrate
    let time_per_block_seconds = (current_difficulty * HASHES_PER_DIFFICULTY) / hashrate;
    let total_time_seconds = blocks_needed * time_per_block_seconds;
    let time_required_hours = total_time_seconds / 3600.0;
    let time_required_days = total_time_seconds / SECONDS_PER_DAY;
    
    // Calculate hashrate required for target time
    let target_seconds = target_days * SECONDS_PER_DAY;
    let hashrate_required = (blocks_needed * current_difficulty * HASHES_PER_DIFFICULTY) / target_seconds;
    
    Ok(ReorgCalculation {
        fork_height,
        current_height,
        blocks_to_reorg,
        total_work,
        current_difficulty,
        blocks_needed,
        time_required_hours,
        time_required_days,
        hashrate_required,
        timestamp: Utc::now(),
    })
}

fn find_viable_target_heights(client: &Client, hashrate: f64, max_days: f64) -> Result<Vec<u64>> {
    let current_height = client.get_block_count()?;
    let mut viable_heights = Vec::new();
    
    // Test various fork heights going back in time
    let test_heights = [
        current_height.saturating_sub(1),
        current_height.saturating_sub(10),
        current_height.saturating_sub(50),
        current_height.saturating_sub(100),
        current_height.saturating_sub(500),
        current_height.saturating_sub(1000),
        current_height.saturating_sub(5000),
    ];
    
    for &height in &test_heights {
        if height > 0 {
            match calculate_reorg_requirements(client, height, hashrate, max_days) {
                Ok(calc) => {
                    if calc.time_required_days <= max_days {
                        viable_heights.push(height);
                    }
                }
                Err(e) => {
                    println!("Warning: Failed to calculate for height {}: {}", height, e);
                }
            }
        }
    }
    
    Ok(viable_heights)
}

fn format_hashrate(hashrate: f64) -> String {
    if hashrate >= 1e15 {
        format!("{:.2} PH/s", hashrate / 1e15)
    } else if hashrate >= 1e12 {
        format!("{:.2} TH/s", hashrate / 1e12)
    } else if hashrate >= 1e9 {
        format!("{:.2} GH/s", hashrate / 1e9)
    } else {
        format!("{:.0} H/s", hashrate)
    }
}

fn display_calculation(calc: &ReorgCalculation, provided_hashrate: f64) {
    println!("\n=== Testnet4 Reorg Calculation ===");
    println!("Timestamp: {}", calc.timestamp.format("%Y-%m-%d %H:%M:%S UTC"));
    println!("Fork Height: {}", calc.fork_height);
    println!("Current Height: {}", calc.current_height);
    println!("Blocks to Reorg: {}", calc.blocks_to_reorg);
    println!("Total Existing Chain Work: {:.2}", calc.total_work);
    println!("Current Difficulty: {:.2}", calc.current_difficulty);
    println!("New Chain Blocks Needed: {:.0}", calc.blocks_needed);
    println!();
    println!("=== With Your Hashrate ({}) ===", format_hashrate(provided_hashrate));
    println!("Time Required: {:.2} hours ({:.2} days)", calc.time_required_hours, calc.time_required_days);
    println!();
    println!("=== For Target Time (3 days) ===");
    println!("Hashrate Required: {}", format_hashrate(calc.hashrate_required));
    
    if calc.blocks_needed <= 1.0 {
        println!("\nNote: A single high-difficulty block may suffice due to Testnet4's 20-minute rule.");
    }
}

fn save_to_file(calculations: &[ReorgCalculation], filename: &str, provided_hashrate: f64) -> Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(filename)
        .context("Failed to open output file")?;
    
    writeln!(file, "\n=== Testnet4 Reorg Calculations - {} ===", Utc::now().format("%Y-%m-%d %H:%M:%S UTC"))?;
    
    for calc in calculations {
        writeln!(file, "\nFork Height: {}", calc.fork_height)?;
        writeln!(file, "Current Height: {}", calc.current_height)?;
        writeln!(file, "Blocks to Reorg: {}", calc.blocks_to_reorg)?;
        writeln!(file, "Total Work: {:.2}", calc.total_work)?;
        writeln!(file, "Current Difficulty: {:.2}", calc.current_difficulty)?;
        writeln!(file, "Blocks Needed: {:.0}", calc.blocks_needed)?;
        writeln!(file, "Time Required ({}): {:.2} days", format_hashrate(provided_hashrate), calc.time_required_days)?;
        writeln!(file, "Hashrate for 3 days: {}", format_hashrate(calc.hashrate_required))?;
        writeln!(file, "Timestamp: {}", calc.timestamp.format("%Y-%m-%d %H:%M:%S UTC"))?;
        writeln!(file, "---")?;
    }
    
    println!("Results saved to: {}", filename);
    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();
    let (_rpc_url, default_user, default_password, default_port, default_hashrate, default_target_days) = load_config()?;
    
    // Override with command line arguments
    let rpc_user = args.rpcuser.unwrap_or(default_user);
    let rpc_password = args.rpcpassword.unwrap_or(default_password);
    let rpc_port = args.rpcport.unwrap_or(default_port);
    let hashrate = args.hashrate.unwrap_or(default_hashrate);
    let target_days = args.target_days.unwrap_or(default_target_days);
    
    let final_rpc_url = format!("http://127.0.0.1:{}", rpc_port);
    let client = connect_to_node(&final_rpc_url, &rpc_user, &rpc_password)?;
    
    println!("Connected to Testnet4 node at {}", final_rpc_url);
    let current_height = client.get_block_count()?;
    println!("Current block height: {}", current_height);
    
    // Get chain info more safely
    match client.get_blockchain_info() {
        Ok(info) => println!("Chain: {}", info.chain),
        Err(_) => println!("Chain: testnet4 (detected)")
    };
    
    let mut calculations = Vec::new();
    
    if args.batch_calculate {
        println!("\nFinding viable target heights for {} within {} days...", format_hashrate(hashrate), target_days);
        let viable_heights = find_viable_target_heights(&client, hashrate, target_days)?;
        
        if viable_heights.is_empty() {
            println!("No viable target heights found within {} days with {}", target_days, format_hashrate(hashrate));
        } else {
            println!("Found {} viable target heights:", viable_heights.len());
            for &height in &viable_heights {
                let calc = calculate_reorg_requirements(&client, height, hashrate, target_days)?;
                display_calculation(&calc, hashrate);
                calculations.push(calc);
            }
        }
    } else if let Some(fork_height) = args.fork_height {
        let calc = calculate_reorg_requirements(&client, fork_height, hashrate, target_days)?;
        display_calculation(&calc, hashrate);
        calculations.push(calc);
    } else {
        // Default: calculate for a recent block that should be viable
        let current_height = client.get_block_count()?;
        let suggested_height = current_height.saturating_sub(100); // Go back 100 blocks
        
        println!("\nNo fork height specified. Calculating for suggested height: {}", suggested_height);
        let calc = calculate_reorg_requirements(&client, suggested_height, hashrate, target_days)?;
        display_calculation(&calc, hashrate);
        calculations.push(calc);
        
        println!("\nTo calculate for a specific height, use: --fork-height <height>");
        println!("To find all viable heights, use: --batch-calculate");
    }
    
    // Save results
    let output_file = env::var("OUTPUT_FILE").unwrap_or_else(|_| "reorg_calculations.txt".to_string());
    save_to_file(&calculations, &output_file, hashrate)?;
    
    Ok(())
}