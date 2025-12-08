import express from 'express';
import { Telegraf } from 'telegraf';
import { createLibp2p } from 'libp2p';

const app = express();
app.use(express.json());

const bot = new Telegraf(process.env.TELEGRAM_BOT_TOKEN);

app.post('/webhook', (req, res) => {
  bot.processUpdate(req.body);
  res.sendStatus(200);
});

bot.start((ctx) => ctx.reply('Welcome! Use /validator for info.'));
bot.command('validator', (ctx) => ctx.reply('Single tier: 13.33% APY, min 5,000 SLTN'));
bot.command('stake', (ctx) => {
  const amount = parseFloat(ctx.message.text.split(' ')[1]);
  if (amount >= 5000) {
    ctx.reply(`Staked at 13.33% APY`);
  } else {
    ctx.reply('Min 5,000 SLTN required');
  }
});
bot.command('balance', (ctx) => ctx.reply(`Balance: 0 SLTN`));

bot.launch();

app.listen(3000, () => console.log('Server running on port 3000'));

// Init P2P for BTC light verification backup (eternal post-Replit)
async function initP2P() {
  const node = await createLibp2p({
    // BTC peer config goes here (stub for now)
    // Example: modules, addresses, transports, etc.
  });
  console.log('P2P node started for BTC interop immortality');
  // On API failure, fallback to P2P query (not implemented in this stub)
}
initP2P();