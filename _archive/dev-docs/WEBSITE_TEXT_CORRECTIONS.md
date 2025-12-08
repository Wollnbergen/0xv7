# Website Text Corrections

## CURRENT (INCORRECT):
> "100+ shards, 200,000+ TPS"

---

## CORRECTED (ACCURATE):

### Version 1: Concise
> **"8 shards at launch, expandable to 8,000 shards. Delivering 64,000 TPS initially with capacity to scale to 64 million TPS as demand grows."**

### Version 2: Technical Detail
> **"Launch Configuration: 8 shards processing 8,000 transactions per second each, delivering 64,000 TPS with sub-3 second finality. Auto-expansion triggers at 80% load, doubling shard count (8→16→32→64...) up to 8,000 shards maximum, providing 64 million TPS at full scale."**

### Version 3: Marketing-Focused
> **"Sultan launches with 8 high-performance shards delivering 64,000 TPS. As your network grows, automatic expansion seamlessly scales to 8,000 shards supporting 64 million transactions per second - all while maintaining sub-3 second finality."**

### Version 4: Ecosystem Page
> **"Scalable Architecture: Starting at 8 shards (64,000 TPS), Sultan's automatic expansion grows with demand. Each doubling event adds capacity: 16 shards (128K TPS), 32 shards (256K TPS), 64 shards (512K TPS), continuing up to 8,000 shards delivering 64 million TPS."**

---

## Technical Breakdown (for FAQ/Docs):

**Q: How many shards at launch?**  
A: 8 shards, each processing 8,000 transactions per second.

**Q: What's the total TPS at launch?**  
A: 64,000 TPS (8 shards × 8,000 TPS/shard)

**Q: How does expansion work?**  
A: When any shard reaches 80% load, the network automatically doubles shard count. Expansion is:
- **Automatic**: No manual intervention required
- **Safe**: Zero data loss, all accounts preserved
- **Fast**: <50ms expansion time
- **Idempotent**: Can be triggered repeatedly without issues

**Q: What's the maximum capacity?**  
A: 8,000 shards × 8,000 TPS/shard = **64,000,000 TPS** (64 million)

**Q: How long does expansion take?**  
A: <50ms with zero downtime or data loss

**Q: Is finality affected by shard count?**  
A: No. Finality remains sub-3 seconds regardless of shard count:
- Block time: 2 seconds
- Propagation: <1 second
- Total: <3 seconds finality

---

## Expansion Progression Table (for Technical Docs):

| Shards | TPS Capacity | Trigger Load | Status |
|--------|--------------|--------------|--------|
| 8      | 64,000       | 51,200 TPS   | Launch ✅ |
| 16     | 128,000      | 102,400 TPS  | Auto   |
| 32     | 256,000      | 204,800 TPS  | Auto   |
| 64     | 512,000      | 409,600 TPS  | Auto   |
| 128    | 1,024,000    | 819,200 TPS  | Auto   |
| 256    | 2,048,000    | 1.6M TPS     | Auto   |
| 512    | 4,096,000    | 3.3M TPS     | Auto   |
| 1,024  | 8,192,000    | 6.6M TPS     | Auto   |
| 2,048  | 16,384,000   | 13.1M TPS    | Auto   |
| 4,096  | 32,768,000   | 26.2M TPS    | Auto   |
| 8,000  | 64,000,000   | Max Capacity | Cap    |

*Each row represents a doubling event triggered at 80% load of previous capacity*

---

## Recommended Replacement:

**Replace this paragraph:**
> "Sultan's sharding architecture delivers over 100 shards processing 200,000+ transactions per second with instant finality."

**With this paragraph:**
> "Sultan launches with 8 high-performance shards processing 64,000 transactions per second. As network demand grows, automatic expansion seamlessly scales to 8,000 shards supporting 64 million transactions per second - all while maintaining sub-3 second finality. Expansion is triggered automatically at 80% load, ensuring the network always has headroom for growth."

---

*Last Updated: $(date)*  
*Configuration: sultan-core/src/sharding_production.rs*  
*Verified: EXPANSION_TESTING_REPORT.md*
