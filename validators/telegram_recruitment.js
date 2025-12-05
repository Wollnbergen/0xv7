const { Telegraf } = require('telegraf');

// Initialize bot (use actual token in production)
const bot = new Telegraf(process.env.BOT_TOKEN || 'test_token');

// Store validator applications
const applications = new Map();
const approved = new Map();

bot.start((ctx) => {
    ctx.reply(`
🚀 Welcome to Sultan Chain Validator Program!

💰 Earn 26.67% APY (37.33% on mobile!)
⚡ Zero gas fees forever
🔒 Minimum stake: 5,000 SLTN

Commands:
/apply - Apply to become a validator
/status - Check your application status
/validators - See current validators
/help - Get help
    `);
});

bot.command('apply', (ctx) => {
    const userId = ctx.from.id;
    
    if (applications.has(userId) || approved.has(userId)) {
        ctx.reply('You already have an application on file.');
        return;
    }
    
    ctx.reply(`
📋 Validator Application

Please answer these questions:

1. How much SLTN will you stake? (minimum 5,000)
2. What's your server location?
3. Are you using mobile? (yes/no)

Reply with your answers like this:
/stake 10000 Europe yes
    `);
});

bot.command('stake', (ctx) => {
    const userId = ctx.from.id;
    const args = ctx.message.text.split(' ');
    
    if (args.length < 4) {
        ctx.reply('Please provide: amount, location, mobile (yes/no)');
        return;
    }
    
    const amount = parseInt(args[1]);
    const location = args[2];
    const mobile = args[3].toLowerCase() === 'yes';
    
    if (amount < 5000) {
        ctx.reply('❌ Minimum stake is 5,000 SLTN');
        return;
    }
    
    applications.set(userId, {
        username: ctx.from.username,
        stake: amount,
        location: location,
        mobile: mobile,
        applied: new Date()
    });
    
    const apy = mobile ? 37.33 : 26.67;
    const dailyRewards = (amount * (apy / 100) / 365).toFixed(2);
    
    ctx.reply(`
✅ Application Received!

📊 Your Details:
• Stake: ${amount.toLocaleString()} SLTN
• Location: ${location}
• APY: ${apy}%
• Daily Rewards: ~${dailyRewards} SLTN
• Mobile Bonus: ${mobile ? 'YES' : 'NO'}

⏳ Status: PENDING APPROVAL

You'll be notified within 24 hours.
Join @SultanValidators for updates!
    `);
});

bot.command('status', (ctx) => {
    const userId = ctx.from.id;
    
    if (approved.has(userId)) {
        const validator = approved.get(userId);
        ctx.reply(`
✅ Status: APPROVED & ACTIVE

Your validator is earning rewards!
• Daily: ${(validator.stake * 0.2667 / 365).toFixed(2)} SLTN
• Address: sultan1${userId}
        `);
    } else if (applications.has(userId)) {
        ctx.reply('⏳ Status: PENDING APPROVAL');
    } else {
        ctx.reply('No application found. Use /apply to start!');
    }
});

bot.command('validators', (ctx) => {
    const validatorCount = applications.size + approved.size + 3; // +3 genesis
    const totalStaked = Array.from(applications.values())
        .concat(Array.from(approved.values()))
        .reduce((sum, v) => sum + v.stake, 300000); // 300k from genesis
    
    ctx.reply(`
🌐 Sultan Chain Validators

👥 Active Validators: ${validatorCount}
💎 Total Staked: ${totalStaked.toLocaleString()} SLTN
📈 Network APY: 26.67%
📱 Mobile APY: 37.33%
⚡ Gas Fees: $0.00

Apply now with /apply!
    `);
});

// Admin commands (for approving validators)
bot.command('approve', (ctx) => {
    // In production, check if admin
    const args = ctx.message.text.split(' ');
    const username = args[1];
    
    for (const [userId, app] of applications) {
        if (app.username === username) {
            approved.set(userId, app);
            applications.delete(userId);
            ctx.reply(`✅ Approved validator: ${username}`);
            return;
        }
    }
    
    ctx.reply('Application not found');
});

console.log('🤖 Sultan Chain Recruitment Bot Started!');
console.log('Add actual bot token to activate');

// Export for testing
module.exports = bot;
