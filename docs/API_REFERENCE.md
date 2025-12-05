# Sultan Chain API Reference

## Base URL
https://api.sultanchain.io/v1

## Authentication
All API calls are FREE - no API keys required (zero fees!)

## Endpoints

### Send Transaction
```http
POST /transaction
Content-Type: application/json

{
  "from": "sultan1abc...",
  "to": "sultan1xyz...",
  "amount": 100,
  "memo": "optional"
}

Response:
{
  "hash": "0x...",
  "gas_fee": 0.00,
  "status": "confirmed"
}
Rate Limits
No rate limits (we want maximum usage!)
1.2M+ TPS capacity
Fees
ALL endpoints: $0.00 forever
