# Block Invalidation Instructions for Bitcoin Testnet4

## Overview
Block invalidation is used to remove blocks from your local node's view of the blockchain, forcing it to reorganize. This is useful for testing reorgs or when you want to exclude certain blocks.

## Prerequisites
- Bitcoin Core node running with Testnet4 (`-testnet4` flag)
- RPC access enabled with credentials
- `bitcoin-cli` installed (comes with Bitcoin Core)

## Basic Commands

### 1. Invalidate a Block
To invalidate a specific block by its hash:

```bash
bitcoin-cli -testnet4 -rpcuser=myusername -rpcpassword=mypassword -rpcport=48337 invalidateblock <block_hash>
```

### 2. Get Block Hash by Height
If you know the block height but need the hash:

```bash
bitcoin-cli -testnet4 -rpcuser=myusername -rpcpassword=mypassword -rpcport=48337 getblockhash <block_height>
```

### 3. Reconsider a Block (Undo Invalidation)
To undo a block invalidation:

```bash
bitcoin-cli -testnet4 -rpcuser=myusername -rpcpassword=mypassword -rpcport=48337 reconsiderblock <block_hash>
```

## Step-by-Step Process

### Step 1: Find the Target Block
First, identify the block you want to invalidate. You can use the reorg calculator to find a suitable fork point:

```bash
# Get current blockchain info
bitcoin-cli -testnet4 -rpcuser=myusername -rpcpassword=mypassword -rpcport=48337 getblockchaininfo

# Get block hash for a specific height
bitcoin-cli -testnet4 -rpcuser=myusername -rpcpassword=mypassword -rpcport=48337 getblockhash 100000
```

### Step 2: Invalidate the Block
Once you have the block hash, invalidate it:

```bash
# Example: Invalidate block at height 100000
BLOCK_HASH=$(bitcoin-cli -testnet4 -rpcuser=myusername -rpcpassword=mypassword -rpcport=48337 getblockhash 100000)
bitcoin-cli -testnet4 -rpcuser=myusername -rpcpassword=mypassword -rpcport=48337 invalidateblock $BLOCK_HASH
```

### Step 3: Verify the Invalidation
Check that the blockchain has reorganized:

```bash
# Check new chain tip
bitcoin-cli -testnet4 -rpcuser=myusername -rpcpassword=mypassword -rpcport=48337 getblockchaininfo

# Verify the invalidated block is no longer in the active chain
bitcoin-cli -testnet4 -rpcuser=myusername -rpcpassword=mypassword -rpcport=48337 getblock $BLOCK_HASH
```

## Important Notes

### Effects of Block Invalidation
- **Local Only**: Block invalidation only affects your local node, not the network
- **Cascade Effect**: Invalidating a block also invalidates all blocks built on top of it
- **Mempool Reset**: Transactions from invalidated blocks may return to the mempool
- **Reorg Trigger**: The node will reorganize to the best valid chain it knows about

### Safety Considerations
- **Backup**: Always backup your wallet before invalidating blocks
- **Testnet Only**: These examples are for Testnet4 - be extremely cautious on mainnet
- **Reversible**: Use `reconsiderblock` to undo invalidations
- **Peer Effects**: Your node may temporarily diverge from network consensus

### Recovery
If something goes wrong, you can:

1. **Reconsider all invalidated blocks**:
   ```bash
   # You'll need to reconsider in reverse order (most recent first)
   bitcoin-cli -testnet4 -rpcuser=myusername -rpcpassword=mypassword -rpcport=48337 reconsiderblock <block_hash>
   ```

2. **Restart the node** (will clear invalidations):
   ```bash
   # Stop Bitcoin Core
   bitcoin-cli -testnet4 -rpcuser=myusername -rpcpassword=mypassword -rpcport=48337 stop
   
   # Restart with same parameters
   bitcoind -testnet4 -rpcuser=myusername -rpcpassword=mypassword -rpcport=48337
   ```

3. **Force reindex** (nuclear option):
   ```bash
   bitcoind -testnet4 -reindex -rpcuser=myusername -rpcpassword=mypassword -rpcport=48337
   ```

## Using with the Reorg Calculator

After running the reorg calculator to find a suitable fork point:

```bash
# Example: Calculator suggests forking from height 99500
FORK_HEIGHT=99500
FORK_HASH=$(bitcoin-cli -testnet4 -rpcuser=myusername -rpcpassword=mypassword -rpcport=48337 getblockhash $FORK_HEIGHT)

# Invalidate the block after the fork point
NEXT_HEIGHT=$((FORK_HEIGHT + 1))
NEXT_HASH=$(bitcoin-cli -testnet4 -rpcuser=myusername -rpcpassword=mypassword -rpcport=48337 getblockhash $NEXT_HEIGHT)
bitcoin-cli -testnet4 -rpcuser=myusername -rpcpassword=mypassword -rpcport=48337 invalidateblock $NEXT_HASH

echo "Invalidated block $NEXT_HASH at height $NEXT_HEIGHT"
echo "Chain should now fork from height $FORK_HEIGHT"
```

## Automation Script

Create a helper script `invalidate_from_height.sh`:

```bash
#!/bin/bash
if [ $# -ne 1 ]; then
    echo "Usage: $0 <fork_height>"
    exit 1
fi

FORK_HEIGHT=$1
NEXT_HEIGHT=$((FORK_HEIGHT + 1))

echo "Getting block hash for height $NEXT_HEIGHT..."
NEXT_HASH=$(bitcoin-cli -testnet4 -rpcuser=myusername -rpcpassword=mypassword -rpcport=48337 getblockhash $NEXT_HEIGHT)

echo "Invalidating block $NEXT_HASH..."
bitcoin-cli -testnet4 -rpcuser=myusername -rpcpassword=mypassword -rpcport=48337 invalidateblock $NEXT_HASH

echo "Done! Chain should now fork from height $FORK_HEIGHT"
echo "Current chain tip:"
bitcoin-cli -testnet4 -rpcuser=myusername -rpcpassword=mypassword -rpcport=48337 getblockchaininfo | grep blocks
```

Make it executable and use:
```bash
chmod +x invalidate_from_height.sh
./invalidate_from_height.sh 99500
```