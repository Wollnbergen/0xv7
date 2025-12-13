const express = require('express');
const rateLimit = require('express-rate-limit');

// Create different limiters for different endpoints
const transferLimiter = rateLimit({
    windowMs: 1 * 60 * 1000, // 1 minute
    max: 10, // 10 transfers per minute
    message: 'Too many transfers, please try again later',
    standardHeaders: true,
    legacyHeaders: false,
});

const stakingLimiter = rateLimit({
    windowMs: 1 * 60 * 1000,
    max: 5, // 5 staking operations per minute
    message: 'Too many staking operations',
});

const queryLimiter = rateLimit({
    windowMs: 1 * 60 * 1000,
    max: 100, // 100 queries per minute
    message: 'Too many queries',
});

// Apply to Sultan Chain API
const app = express();

app.use('/api/transfer', transferLimiter);
app.use('/api/stake', stakingLimiter);
app.use('/api/query', queryLimiter);

console.log('✅ Rate limiting configured');
