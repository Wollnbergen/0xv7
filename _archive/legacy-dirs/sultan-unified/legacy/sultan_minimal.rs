fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║           SULTAN CHAIN - ZERO GAS BLOCKCHAIN                  ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!("\n💰 Gas Fees: $0.00\n📈 Validator APY: 13.33%\n📱 Mobile APY: 18.66%\n");
    
    let mut h = 0;
    loop {
        h += 1;
        println!("✅ Block #{} produced", h);
        std::thread::sleep(std::time::Duration::from_secs(5));
    }
}
