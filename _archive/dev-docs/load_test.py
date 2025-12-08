#!/usr/bin/env python3
"""
Sultan L1 Production Load Test
Real transaction execution with actual TPS measurement
"""

import asyncio
import aiohttp
import time
import random
import json
from datetime import datetime
from typing import List, Dict
import sys

class Colors:
    CYAN = '\033[96m'
    GREEN = '\033[92m'
    YELLOW = '\033[93m'
    RED = '\033[91m'
    BOLD = '\033[1m'
    END = '\033[0m'

class LoadTest:
    def __init__(self, rpc_url: str, total_txs: int, workers: int):
        self.rpc_url = rpc_url
        self.total_txs = total_txs
        self.workers = workers
        self.successful = 0
        self.failed = 0
        self.start_time = None
        self.end_time = None
        self.accounts = [f"account_{i}" for i in range(1000)]
        
    async def check_node_status(self, session: aiohttp.ClientSession) -> Dict:
        """Check if node is running and get status"""
        async with session.get(f"{self.rpc_url}/status") as response:
            if response.status != 200:
                raise Exception(f"Node not responding: {response.status}")
            return await response.json()
    
    async def submit_transaction(self, session: aiohttp.ClientSession, tx: Dict) -> bool:
        """Submit a single transaction"""
        try:
            async with session.post(
                f"{self.rpc_url}/tx",
                json=tx,
                timeout=aiohttp.ClientTimeout(total=5)
            ) as response:
                if response.status == 200:
                    self.successful += 1
                    return True
                else:
                    self.failed += 1
                    return False
        except Exception as e:
            self.failed += 1
            return False
    
    def generate_transaction(self, nonce: int) -> Dict:
        """Generate a random transaction"""
        from_account = random.choice(self.accounts)
        to_account = random.choice(self.accounts)
        while to_account == from_account:
            to_account = random.choice(self.accounts)
        
        return {
            "from": from_account,
            "to": to_account,
            "amount": random.randint(1, 1000),
            "gas_fee": 0,
            "timestamp": int(time.time()),
            "nonce": nonce,
            "signature": f"sig_{nonce}"
        }
    
    async def worker(self, worker_id: int, txs_per_worker: int, session: aiohttp.ClientSession):
        """Worker coroutine to send transactions"""
        nonce = 0
        for _ in range(txs_per_worker):
            tx = self.generate_transaction(nonce)
            await self.submit_transaction(session, tx)
            nonce += 1
            
            if (nonce % 100) == 0:
                tps = self.get_current_tps()
                print(f"Worker {worker_id:03d}: {nonce}/{txs_per_worker} txs | Current TPS: {tps:.2f}", end='\r')
    
    def get_current_tps(self) -> float:
        """Calculate current TPS"""
        if self.start_time is None:
            return 0.0
        elapsed = time.time() - self.start_time
        if elapsed > 0:
            return self.successful / elapsed
        return 0.0
    
    def print_header(self):
        """Print test header"""
        print()
        print(f"{Colors.CYAN}{'═' * 80}{Colors.END}")
        print(f"{Colors.CYAN}{Colors.BOLD}{'  🚀 SULTAN L1 PRODUCTION LOAD TEST 🚀':^80}{Colors.END}")
        print(f"{Colors.CYAN}{'═' * 80}{Colors.END}")
        print()
    
    def print_config(self):
        """Print test configuration"""
        print(f"  {Colors.BOLD}RPC Endpoint:{Colors.END}       {Colors.CYAN}{self.rpc_url}{Colors.END}")
        print(f"  {Colors.BOLD}Total Transactions:{Colors.END} {Colors.YELLOW}{Colors.BOLD}{self.total_txs:,}{Colors.END}")
        print(f"  {Colors.BOLD}Concurrent Workers:{Colors.END} {Colors.YELLOW}{Colors.BOLD}{self.workers}{Colors.END}")
        print(f"  {Colors.BOLD}Test Accounts:{Colors.END}      {Colors.CYAN}{len(self.accounts)}{Colors.END}")
        print()
    
    def print_results(self):
        """Print test results"""
        elapsed = self.end_time - self.start_time
        total = self.successful + self.failed
        tps = self.successful / elapsed if elapsed > 0 else 0
        success_rate = (self.successful / total * 100) if total > 0 else 0
        
        print()
        print(f"{Colors.CYAN}{'═' * 80}{Colors.END}")
        print(f"{Colors.CYAN}{Colors.BOLD}  LOAD TEST RESULTS{Colors.END}")
        print(f"{Colors.CYAN}{'═' * 80}{Colors.END}")
        print()
        print(f"  {Colors.BOLD}Total Transactions:{Colors.END}  {Colors.YELLOW}{Colors.BOLD}{total:,}{Colors.END}")
        print(f"  {Colors.GREEN}Successful:{Colors.END}          {Colors.GREEN}{Colors.BOLD}{self.successful:,}{Colors.END}")
        print(f"  {Colors.RED}Failed:{Colors.END}              {Colors.RED}{Colors.BOLD}{self.failed:,}{Colors.END}")
        print(f"  {Colors.BOLD}Success Rate:{Colors.END}        {Colors.CYAN}{Colors.BOLD}{success_rate:.2f}%{Colors.END}")
        print()
        print(f"  {Colors.BOLD}Total Time:{Colors.END}          {Colors.YELLOW}{Colors.BOLD}{elapsed:.2f}s{Colors.END}")
        print(f"  {Colors.BOLD}Avg Latency:{Colors.END}         {Colors.CYAN}{elapsed * 1000 / total:.2f}ms{Colors.END}")
        print()
        print(f"  {Colors.BOLD}ACTUAL TPS:{Colors.END}          {Colors.GREEN}{Colors.BOLD}{tps:,.2f}{Colors.END}")
        print()
        print(f"{Colors.CYAN}{'═' * 80}{Colors.END}")
        print()
        
        # Comparison
        print(f"{Colors.CYAN}{Colors.BOLD}  COMPARISON WITH OTHER BLOCKCHAINS{Colors.END}")
        print(f"{'─' * 80}")
        print(f"  {'Ethereum:':<20} {Colors.RED}~15 TPS{Colors.END}")
        print(f"  {'Bitcoin:':<20} {Colors.RED}~7 TPS{Colors.END}")
        print(f"  {'Solana:':<20} {Colors.YELLOW}~3,000 TPS{Colors.END}")
        print(f"  {'Avalanche:':<20} {Colors.YELLOW}~4,500 TPS{Colors.END}")
        print(f"  {Colors.BOLD}{'Sultan L1:':<20}{Colors.END} {Colors.GREEN}{Colors.BOLD}{tps:,.2f} TPS{Colors.END}")
        print()
        
        if tps > 10000:
            multiplier = tps / 3000
            print(f"  {Colors.YELLOW}🚀{Colors.END} Sultan L1 is {Colors.GREEN}{Colors.BOLD}{multiplier:.1f}x{Colors.END} faster than Solana!")
        elif tps > 1000:
            print(f"  {Colors.GREEN}✅{Colors.END} Excellent performance!")
        else:
            print(f"  {Colors.YELLOW}⚠️{Colors.END} Performance below expected (target: 200,000 TPS)")
        print()
    
    async def run(self):
        """Execute the load test"""
        self.print_header()
        self.print_config()
        
        async with aiohttp.ClientSession() as session:
            # Check node status
            print(f"  {Colors.CYAN}📡{Colors.END} Checking node status...")
            try:
                status = await self.check_node_status(session)
                print(f"  {Colors.GREEN}✅{Colors.END} Node is online!")
                print(f"    • Block Height: {Colors.CYAN}{status['height']}{Colors.END}")
                print(f"    • Validators: {Colors.CYAN}{status.get('validator_count', 0)}{Colors.END}")
                sharding = status.get('sharding_enabled', False)
                if sharding:
                    shard_count = status.get('shard_count', 0)
                    print(f"    • Sharding: {Colors.GREEN}✅ {shard_count} shards{Colors.END}")
                else:
                    print(f"    • Sharding: {Colors.RED}❌ Disabled{Colors.END}")
                print()
            except Exception as e:
                print(f"  {Colors.RED}❌ Error: {e}{Colors.END}")
                return
            
            # Calculate transactions per worker
            txs_per_worker = self.total_txs // self.workers
            
            print(f"  {Colors.YELLOW}🔥{Colors.END} Starting load test...")
            print(f"    • {Colors.CYAN}{txs_per_worker:,}{Colors.END} transactions per worker")
            print()
            
            # Start timing
            self.start_time = time.time()
            
            # Create worker tasks
            tasks = []
            for worker_id in range(self.workers):
                task = asyncio.create_task(
                    self.worker(worker_id, txs_per_worker, session)
                )
                tasks.append(task)
            
            # Wait for all workers to complete
            await asyncio.gather(*tasks)
            
            # End timing
            self.end_time = time.time()
            
            print()  # Clear the progress line
            print(f"  {Colors.GREEN}✅{Colors.END} All workers completed!")
            
            # Print results
            self.print_results()
            
            # Post-test verification
            print(f"{Colors.CYAN}{Colors.BOLD}  POST-TEST VERIFICATION{Colors.END}")
            print(f"{'─' * 80}")
            final_status = await self.check_node_status(session)
            print(f"  • Final Block Height: {Colors.CYAN}{final_status['height']}{Colors.END}")
            print(f"  • Pending Transactions: {Colors.YELLOW}{final_status.get('pending_txs', 0)}{Colors.END}")
            print(f"  • Total Accounts: {Colors.CYAN}{final_status.get('total_accounts', 0)}{Colors.END}")
            print()
            
            print(f"{Colors.GREEN}{Colors.BOLD}🎉 Load test completed successfully!{Colors.END}")
            print()

async def main():
    # Configuration from environment or defaults
    import os
    rpc_url = os.getenv('RPC_URL', 'http://localhost:26657')
    total_txs = int(os.getenv('TOTAL_TXS', '10000'))
    workers = int(os.getenv('WORKERS', '100'))
    
    test = LoadTest(rpc_url, total_txs, workers)
    await test.run()

if __name__ == '__main__':
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        print(f"\n{Colors.YELLOW}Test interrupted by user{Colors.END}")
        sys.exit(1)
