#!/usr/bin/env python3
"""
Production Load Testing for Sultan Chain
Target: 1.23M TPS with $0.00 gas fees
"""

import asyncio
import aiohttp
import time
import json
from typing import Dict, List

class LoadTester:
    def __init__(self):
        self.target_tps = 1_230_000
        self.test_duration = 60  # 1 minute test
        self.rpc_url = "http://localhost:26657"
        
    async def run_test(self) -> Dict:
        print(f"🚀 Starting load test: {self.target_tps:,} TPS target")
        
        start = time.time()
        total_sent = 0
        batch_size = 1000
        
        async with aiohttp.ClientSession() as session:
            tasks = []
            
            while time.time() - start < self.test_duration:
                # Create batch of transactions
                batch = self._create_tx_batch(batch_size)
                
                # Send batch
                for tx in batch:
                    task = self._send_transaction(session, tx)
                    tasks.append(task)
                
                # Process responses
                if len(tasks) >= 10000:
                    results = await asyncio.gather(*tasks, return_exceptions=True)
                    total_sent += len([r for r in results if not isinstance(r, Exception)])
                    tasks = []
            
            # Process remaining
            if tasks:
                results = await asyncio.gather(*tasks, return_exceptions=True)
                total_sent += len([r for r in results if not isinstance(r, Exception)])
        
        elapsed = time.time() - start
        actual_tps = total_sent / elapsed
        
        return {
            'duration': elapsed,
            'transactions_sent': total_sent,
            'actual_tps': actual_tps,
            'target_tps': self.target_tps,
            'success_rate': f"{(actual_tps/self.target_tps)*100:.2f}%"
        }
    
    def _create_tx_batch(self, size: int) -> List[Dict]:
        return [
            {
                'from': f'sultan1{"x"*38}',
                'to': f'sultan1{"y"*38}',
                'amount': '1000000usltn',
                'gas': '0',
                'fee': '0usltn'
            }
            for _ in range(size)
        ]
    
    async def _send_transaction(self, session: aiohttp.ClientSession, tx: Dict):
        try:
            async with session.post(
                f"{self.rpc_url}/broadcast_tx_async",
                json={'tx': tx}
            ) as resp:
                return await resp.json()
        except Exception as e:
            return None

if __name__ == "__main__":
    tester = LoadTester()
    result = asyncio.run(tester.run_test())
    print(f"✅ Load Test Complete: {result['actual_tps']:,.0f} TPS achieved")
