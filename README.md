# Testnet4 Reorg Calculator

A Rust tool to calculate proof-of-work requirements for reorganizing Bitcoin Testnet4 blockchain from any block height.

## Features

- Calculate work needed to reorg from any block height
- Find viable target heights within your timeframe
- Support for .env configuration with CLI overrides
- Output results to file for future reference
- 1 PH/s default hashrate with easy customization
- Block invalidation instructions included

## Quick Start

1. **Build the project**:
   ```bash
   cargo build --release
   ```

2. **Run with default settings** (suggests a viable height):
   ```bash
   cargo run
   ```

3. **Calculate for specific height**:
   ```bash
   cargo run -- --fork-height 100000
   ```

4. **Find all viable heights** (completable in 3 days with 1 PH/s):
   ```bash
   cargo run -- --batch-calculate
   ```

## Configuration

Edit `.env` file to change defaults:
```env
RPC_URL=http://127.0.0.1:48337
RPC_USER=myusername
RPC_PASSWORD=mypassword
RPC_PORT=48337
DEFAULT_HASHRATE=1000000000000000  # 1 PH/s
TARGET_DAYS=3
OUTPUT_FILE=reorg_calculations.txt
```

## Command Line Options

- `--fork-height <height>`: Specific block height to fork from
- `--target-days <days>`: Target completion time (default: 3)
- `--hashrate <hashes/sec>`: Your available hashrate (default: 1 PH/s)
- `--rpcuser <username>`: Override RPC username
- `--rpcpassword <password>`: Override RPC password
- `--rpcport <port>`: Override RPC port
- `--batch-calculate`: Find all viable target heights

## Examples

```bash
# Calculate with 500 TH/s for 2 days
cargo run -- --hashrate 500000000000000 --target-days 2

# Override RPC settings
cargo run -- --rpcuser alice --rpcpassword secret123 --rpcport 18332

# Find heights achievable in 1 day with 2 PH/s
cargo run -- --batch-calculate --hashrate 2000000000000000 --target-days 1
```

## Output

Results are displayed on screen and saved to `reorg_calculations.txt`:

```
=== Testnet4 Reorg Calculation ===
Fork Height: 99500
Current Height: 100000
Blocks to Reorg: 501
Total Existing Chain Work: 5000000.50
Current Difficulty: 10000.00
New Chain Blocks Needed: 500

=== With Your Hashrate (1.00 PH/s) ===
Time Required: 61.44 hours (2.56 days)

=== For Target Time (3 days) ===
Hashrate Required: 853.33 TH/s
```

## Block Invalidation

See `INVALIDATE_BLOCKS.md` for detailed instructions on how to invalidate blocks via RPC to trigger reorgs.

## Requirements

- Bitcoin Core node running with `-testnet4`
- Rust (latest stable)
- RPC access configured