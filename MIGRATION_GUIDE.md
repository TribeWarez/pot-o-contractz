# Migration Guide: v0.1.x → v0.2.x

## Overview

This guide explains how to upgrade from pot-o-contractz v0.1.x to v0.2.x. **The good news**: Full ABI backward compatibility means **zero breaking changes** and **no account migration needed**.

## Key Changes in v0.2.0

### What's New?

1. **Dependency Injection (DI) Service Architecture**
   - Abstract service traits for business logic
   - Multiple implementations (Legacy v0.1.x + TensorAware v0.2.0)
   - ServiceRegistry DI container for runtime selection

2. **Tensor Network Integration (REALMS Part IV)**
   - Quantum-inspired reward calculations
   - Entropy-based probability functions
   - Device coherence multipliers

3. **Event-Driven State Changes**
   - All state modifications emit events
   - Off-chain systems can track on-chain changes
   - Helps with monitoring and alerting

### What's NOT Changing?

- ✓ **Instruction signatures**: Identical to v0.1.x
- ✓ **Account layouts**: New fields appended only
- ✓ **Default behavior**: Legacy mode produces v0.1.x results
- ✓ **Client compatibility**: v0.1.x clients can still read v0.2.0 accounts

## Upgrade Steps

### 1. Deploy v0.2.0 Programs (Parallel Deployment)

Deploy v0.2.0 programs alongside existing v0.1.x programs initially:

```bash
# 1a. Update your Cargo.toml to use v0.2.x
[dependencies]
tribewarez-pot-o = "0.2.0"
tribewarez-staking = "0.2.1"
tribewarez-vault = "0.2.0"
tribewarez-swap = "0.2.0"
pot-o-core = "0.2.0"

# 1b. Build the new programs
anchor build

# 1c. Deploy to devnet first (recommended)
anchor deploy --provider.cluster devnet

# 1d. Verify deployment
solana program show <program-id> --url devnet
```

### 2. Keep v0.1.x Running (Initial Phase)

Continue running v0.1.x programs in production while testing v0.2.0:

```bash
# In devnet/testnet:
# - v0.1.x programs handle existing client requests
# - v0.2.0 programs running in parallel for testing
# - No migrations needed; accounts work with both versions
```

### 3. Enable Tensor Features (Gradual Rollout)

Enable tensor features progressively per pool/contract:

```bash
# Start with legacy mode (v0.1.x compatible behavior)
solana rpc pot-o config \
  --tenant <POOL_ID> \
  --tensor-enabled 0  # Legacy mode: use v0.1.x logic

# Monitor behavior and metrics
sleep 1 week

# Enable for select pools
solana rpc pot-o config \
  --tenant <POOL_ID> \
  --tensor-enabled 1  # Tensor mode: use v0.2.0 logic

# Validate new rewards and behavior match expectations
# Check entropy calculations, coherence bonuses, etc.

sleep 2 weeks

# Once validated, enable globally
solana rpc pot-o config \
  --global \
  --tensor-enabled 1
```

### 4. Monitor Behavior Changes

#### Key Metrics to Watch

**Rewards (PoT-O)**:
- v0.1.x: Linear per share
- v0.2.0: Tensor-weighted (variance 0-20% based on entropy)

```
Expected change: ±20% variance in individual rewards
Pool total should remain stable
```

**Unlock Probability (Staking)**:
- v0.1.x: Fixed time-lock
- v0.2.0: P(unlock) = tanh(S_A / S_max)

```
Expected change: Dynamic unlock based on pool entropy
Individual unlocks may vary (0-100% time reduction)
```

**Withdrawal Fees (Vault)**:
- v0.1.x: Fixed fee
- v0.2.0: Dynamic (0-50% reduction at max entropy)

```
Expected change: Fees decrease as entropy increases
No additional fees; only reductions
```

**Fee Discounts (Swap)**:
- v0.1.x: Fixed 0.30% swap fee
- v0.2.0: 0-50% discount based on coherence

```
Expected change: Fee reduction for high-coherence users
Pool revenue may decrease 5-15%
```

### 5. Update Client Code (Optional)

Your clients will continue working without changes, but you can optionally use new v0.2.0 features:

#### Rust Client Example

```rust
// v0.1.x code still works exactly the same
use tribewarez_pot_o::PotOClient;

let client = PotOClient::new(connection, program_id);
let miner = client.register_miner(device_type).await?;
let proof = client.submit_proof(proof_params).await?;

// v0.2.0: Now you can also listen to tensor events
use tribewarez_pot_o::events::EntropyStateUpdated;

let subscription = client.subscribe_to_events::<EntropyStateUpdated>()?;
for event in subscription {
    println!("Entropy updated: {:?}", event);
}
```

#### TypeScript/JavaScript Example

```typescript
// v0.1.x code still works
import { PotOClient } from '@tribewarez/pot-o-sdk';

const client = new PotOClient(connection);
await client.registerMiner(deviceType);
const proof = await client.submitProof(proofParams);

// v0.2.0: Subscribe to events
import { EntropyStateUpdated } from '@tribewarez/pot-o-sdk';

const unsubscribe = connection.onLogs(
  minerPubkey,
  (logs) => {
    const event = PotOClient.parseEvent(logs);
    if (event instanceof EntropyStateUpdated) {
      console.log('Entropy updated:', event);
    }
  }
);
```

### 6. Configuration Reference

#### ServiceRegistry Configuration

```rust
// In your contract initialization:
let registry = if config.tensor_enabled != 0 {
    // Use v0.2.0 tensor-aware services
    ServiceRegistry::new_tensor_aware(
        s_max,           // Maximum entropy (1e6 scale)
        bond_dimension,  // Quantum bond dimension (typically 2)
        entropy_weight,  // Weight factor (0.0-1.0)
        max_pool_size,   // Max miners per pool
    )
} else {
    // Use v0.1.x legacy services
    ServiceRegistry::new_legacy()
};
```

#### Device Coherence Factors

```
ASIC:   1.0x (baseline)
GPU:    0.8x
CPU:    0.6x
Mobile: 0.4x

Applied to:
- Reward probability
- Unlock probability calculations
- Fee discount multipliers
```

### 7. Validation Checklist

Before going fully production with v0.2.0:

- [ ] **Deployment**: All programs deployed and verified
- [ ] **Account Compatibility**: v0.2.0 reads existing v0.1.x accounts correctly
- [ ] **Event Emission**: Events being emitted correctly
- [ ] **Reward Distribution**: Tensor-aware rewards calculated correctly
- [ ] **Entropy Tracking**: Entropy and mutual information updated
- [ ] **Fee Calculations**: Fee discounts applied based on coherence
- [ ] **Unlock Probabilities**: Dynamic unlock times working
- [ ] **Monitor Period**: Run v0.2.0 in parallel for 2+ weeks
- [ ] **Client Updates**: Updated SDKs/clients if needed
- [ ] **Rollback Plan**: Have plan to switch back to v0.1.x if needed

## Troubleshooting

### Issue: "Transaction too large" errors

**Cause**: New tensor fields added to state structs

**Solution**:
- Tensor fields appended, not changing instruction encoding
- Errors unlikely unless your app creates very complex transactions
- Split large transactions if needed

### Issue: Rewards varying unexpectedly

**Cause**: Tensor-aware rewards add variance based on entropy

**Solution**:
- Check if `tensor_enabled = 1` in pool config
- Review entropy values in event logs
- Compare to expected variance (0-20% for rewards)
- Switch to legacy mode if needed: `tensor_enabled = 0`

### Issue: Unlock times changing

**Cause**: Tensor mode uses dynamic unlock based on entropy

**Solution**:
- Review unlock probability: P(unlock) = tanh(S_A / S_max)
- Check entropy contributions via events
- Update client UI to show expected unlock time range

### Issue: v0.1.x clients getting "invalid account" errors

**Cause**: Usually not related to v0.2.0 upgrade

**Solution**:
- v0.2.0 appends fields at END of struct
- v0.1.x clients can still read up to their expected size
- If errors occur, verify account initialization
- Check for PDA derivation path changes (none expected)

## Rollback Plan

If v0.2.0 has critical issues:

```bash
# 1. Stop routing new transactions to v0.2.0
# 2. Resume using v0.1.x programs

# 3. Accounts are still readable by both versions
# 4. No data loss or migrations required

# 5. Can retry v0.2.0 upgrade after fixes
```

## Performance Implications

### Compute Budget Changes

- **Legacy Mode**: Identical to v0.1.x (no additional compute)
- **Tensor Mode**: +15-20% additional compute per transaction
  - Entropy calculations: ~5-10% overhead
  - Mutual information: ~5-10% overhead
  - Decision tree selection: ~2-3% overhead

### Storage Changes

- **Per Account**: +60-90 bytes for tensor fields
- **Total**: Minimal impact (state appended, not reallocated)

## Support & Resources

- **Documentation**: See CHANGELOG.md for detailed feature list
- **Tests**: Review tests/ for examples of all v0.2.0 features
- **IDL**: See idl/ directory for complete API specifications
- **Issues**: Report via GitHub issues on the repository

## Timeline Recommendations

```
Week 1-2: Deploy v0.2.0 to devnet, test independently
Week 3-4: Deploy to testnet alongside v0.1.x
Week 5-6: Validate behavior, monitor metrics
Week 7-8: Gradual pool-by-pool tensor enablement
Week 9+:  Monitor production, adjust configuration as needed
```

---

**Questions?** See the main README.md, CHANGELOG.md, or PHASE6_TESTING_SUMMARY.md for more details.
