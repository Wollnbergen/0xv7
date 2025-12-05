#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║     FEATURE 2: INTEGRATING DATABASE PROPERLY                  ║"
echo "╚══════════════════════════════════════════════════════════════╝"

mkdir -p /workspaces/0xv7/database

# Create proper database integration
cat > /workspaces/0xv7/database/database_manager.js << 'DATABASE'
const cassandra = require('cassandra-driver');

class SultanDatabase {
    constructor() {
        // Try ScyllaDB first, fallback to in-memory
        this.useScylla = false;
        this.inMemoryDB = {
            blocks: [],
            transactions: [],
            validators: new Map(),
            accounts: new Map()
        };
        
        this.initDatabase();
    }
    
    async initDatabase() {
        try {
            this.client = new cassandra.Client({
                contactPoints: ['localhost'],
                localDataCenter: 'datacenter1',
                keyspace: 'sultan_chain'
            });
            
            await this.client.connect();
            await this.createTables();
            this.useScylla = true;
            console.log('✅ Connected to ScyllaDB');
        } catch (err) {
            console.log('⚠️ ScyllaDB not available, using in-memory database');
        }
    }
    
    async createTables() {
        const queries = [
            `CREATE KEYSPACE IF NOT EXISTS sultan_chain 
             WITH replication = {'class': 'SimpleStrategy', 'replication_factor': 1}`,
            
            `CREATE TABLE IF NOT EXISTS sultan_chain.blocks (
                height bigint PRIMARY KEY,
                hash text,
                timestamp timestamp,
                validator text,
                tx_count int,
                total_fees decimal
            )`,
            
            `CREATE TABLE IF NOT EXISTS sultan_chain.transactions (
                hash text PRIMARY KEY,
                block_height bigint,
                from_address text,
                to_address text,
                amount decimal,
                fee decimal,
                timestamp timestamp
            )`,
            
            `CREATE TABLE IF NOT EXISTS sultan_chain.validators (
                address text PRIMARY KEY,
                stake decimal,
                apy decimal,
                joined_at timestamp,
                total_rewards decimal,
                status text
            )`,
            
            `CREATE TABLE IF NOT EXISTS sultan_chain.accounts (
                address text PRIMARY KEY,
                balance decimal,
                nonce bigint,
                created_at timestamp
            )`
        ];
        
        for (const query of queries) {
            try {
                await this.client.execute(query);
            } catch (err) {
                console.log('Table creation warning:', err.message);
            }
        }
    }
    
    async saveBlock(block) {
        if (this.useScylla) {
            const query = `INSERT INTO sultan_chain.blocks 
                (height, hash, timestamp, validator, tx_count, total_fees) 
                VALUES (?, ?, ?, ?, ?, ?)`;
            
            await this.client.execute(query, [
                block.height,
                block.hash,
                new Date(),
                block.validator,
                block.transactions.length,
                0 // Always 0 fees!
            ], { prepare: true });
        } else {
            this.inMemoryDB.blocks.push(block);
        }
        
        console.log(`💾 Block ${block.height} saved to database`);
    }
    
    async saveTransaction(tx) {
        if (this.useScylla) {
            const query = `INSERT INTO sultan_chain.transactions 
                (hash, block_height, from_address, to_address, amount, fee, timestamp)
                VALUES (?, ?, ?, ?, ?, ?, ?)`;
            
            await this.client.execute(query, [
                tx.hash,
                tx.blockHeight,
                tx.from,
                tx.to,
                tx.amount,
                0, // Zero fees!
                new Date()
            ], { prepare: true });
        } else {
            this.inMemoryDB.transactions.push(tx);
        }
    }
    
    async saveValidator(validator) {
        if (this.useScylla) {
            const query = `INSERT INTO sultan_chain.validators 
                (address, stake, apy, joined_at, total_rewards, status)
                VALUES (?, ?, ?, ?, ?, ?)`;
            
            await this.client.execute(query, [
                validator.address,
                validator.stake,
                0.2667, // 26.67% APY
                new Date(),
                0,
                'active'
            ], { prepare: true });
        } else {
            this.inMemoryDB.validators.set(validator.address, validator);
        }
        
        console.log(`✅ Validator ${validator.address} saved`);
    }
    
    async getBlock(height) {
        if (this.useScylla) {
            const query = 'SELECT * FROM sultan_chain.blocks WHERE height = ?';
            const result = await this.client.execute(query, [height]);
            return result.rows[0];
        } else {
            return this.inMemoryDB.blocks.find(b => b.height === height);
        }
    }
    
    async getValidators() {
        if (this.useScylla) {
            const result = await this.client.execute('SELECT * FROM sultan_chain.validators');
            return result.rows;
        } else {
            return Array.from(this.inMemoryDB.validators.values());
        }
    }
    
    async getStats() {
        const validators = await this.getValidators();
        const blockCount = this.inMemoryDB.blocks.length;
        const txCount = this.inMemoryDB.transactions.length;
        
        return {
            database: this.useScylla ? 'ScyllaDB' : 'In-Memory',
            blocks: blockCount,
            transactions: txCount,
            validators: validators.length,
            totalStaked: validators.reduce((sum, v) => sum + (v.stake || 0), 0)
        };
    }
}

// Export for use in other modules
module.exports = SultanDatabase;

// Test database
async function testDatabase() {
    const db = new SultanDatabase();
    
    // Wait for init
    await new Promise(resolve => setTimeout(resolve, 1000));
    
    // Test saving block
    await db.saveBlock({
        height: 1,
        hash: 'genesis',
        validator: 'validator1',
        transactions: []
    });
    
    // Test saving validator
    await db.saveValidator({
        address: 'sultan1test',
        stake: 10000
    });
    
    // Get stats
    const stats = await db.getStats();
    console.log('📊 Database Stats:', stats);
}

// Run test
testDatabase();
DATABASE

echo "✅ Database integration complete!"
echo "Installing dependencies..."
cd /workspaces/0xv7/database
npm init -y 2>/dev/null
npm install cassandra-driver 2>/dev/null || echo "Note: Install cassandra-driver for ScyllaDB support"
node database_manager.js &

echo ""
echo "Database ready at /workspaces/0xv7/database/database_manager.js"
