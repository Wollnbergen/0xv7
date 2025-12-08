from flask import Flask, jsonify
from datetime import datetime

app = Flask(__name__)

def get_current_inflation_rate():
    """Calculate current inflation rate based on year"""
    launch_date = datetime(2024, 1, 1)
    current_date = datetime.now()
    years_since_launch = (current_date - launch_date).days / 365
    
    if years_since_launch < 1:
        return 8.0
    elif years_since_launch < 2:
        return 7.0
    elif years_since_launch < 3:
        return 6.0
    elif years_since_launch < 4:
        return 5.0
    else:
        return 4.0

@app.route('/tokenomics')
def tokenomics():
    current_inflation = get_current_inflation_rate()
    
    return jsonify({
        "token": {
            "name": "Sultan Token",
            "symbol": "SLTN",
            "type": "Native",
            "decimals": 6
        },
        "supply": {
            "initial": "500,000,000 SLTN",
            "initial_raw": "500000000000000",
            "current": "502,000,000 SLTN",  # Example with some inflation
            "circulating": "250,000,000 SLTN",
            "staked": "150,000,000 SLTN",
            "dynamic": True
        },
        "inflation": {
            "current_rate": f"{current_inflation}%",
            "schedule": {
                "year_1": "8%",
                "year_2": "7%",
                "year_3": "6%",
                "year_4": "5%",
                "year_5+": "4%"
            },
            "purpose": [
                "Subsidize zero gas fees",
                "Pay validator rewards (13.33% APY)",
                "Fund community treasury",
                "Support development"
            ],
            "distribution": {
                "staking_rewards": "70%",
                "community": "20%",
                "development": "10%"
            }
        },
        "economics": {
            "staking_apy": "13.33%",
            "gas_fees": "$0.00",
            "transaction_cost": "0 SLTN",
            "min_stake": "5,000 SLTN",
            "unbonding_period": "21 days"
        },
        "distribution": {
            "validator_rewards": "200,000,000 SLTN (40%)",
            "development": "100,000,000 SLTN (20%)",
            "community": "100,000,000 SLTN (20%)",
            "liquidity": "50,000,000 SLTN (10%)",
            "team": "50,000,000 SLTN (10%, 4-year vesting)"
        }
    })

if __name__ == '__main__':
    app.run(port=8081)
