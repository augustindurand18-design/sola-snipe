import pino from "pino";
import { Telegraf } from "telegraf";
import dotenv from "dotenv";

dotenv.config();

const pinoLogger = pino({ transport: { target: 'pino-pretty' } });

class BotLogger {
    private bot: Telegraf | null = null;
    private chatId: string | undefined;

    constructor() {
        const token = process.env.TELEGRAM_BOT_TOKEN;
        this.chatId = process.env.TELEGRAM_CHAT_ID;

        if (token && this.chatId) {
            this.bot = new Telegraf(token);
            pinoLogger.info("Telegram bot initialized for notifications.");
        } else {
            pinoLogger.warn("TELEGRAM_BOT_TOKEN or TELEGRAM_CHAT_ID is missing. Telegram notifications disabled.");
        }
    }

    public info(msg: string) {
        pinoLogger.info(msg);
    }

    public warn(msg: string) {
        pinoLogger.warn(msg);
    }

    public error(msg: string) {
        pinoLogger.error(msg);
    }

    private async sendTelegramMsg(msg: string) {
        if (this.bot && this.chatId) {
            try {
                await this.bot.telegram.sendMessage(this.chatId, msg, { parse_mode: 'HTML' });
            } catch (e) {
                pinoLogger.error(`Failed to send Telegram message: ${e}`);
            }
        }
    }

    public async notifyDetection(tokenName: string, liquiditySol: number) {
        const msg = `🟢 <b>Detection</b>\nToken: ${tokenName}\nLiq: ${liquiditySol} SOL`;
        this.info(msg.replace(/\n/g, " | ").replace(/<[^>]*>?/gm, ''));
        await this.sendTelegramMsg(msg);
    }

    public async notifyBuy(
        tokenName: string, 
        t1Ms: number, 
        t2Ms: number, 
        slippagePercent: number, 
        walletBalance: number
    ) {
        const totalLatency = t1Ms + t2Ms;
        const successProb = 100 - slippagePercent; 

        let msg = `🎯 <b>Target</b> : ${tokenName}\n`;
        msg += `⏱️ <b>Latence Totale</b> : ${totalLatency.toFixed(2)} ms (T1: ${t1Ms.toFixed(2)}ms, T2: ${t2Ms.toFixed(2)}ms)\n`;
        msg += `📊 <b>Probabilité Success</b> : ${successProb.toFixed(2)}%\n`;
        msg += `💵 <b>Statut Wallet</b> : ${walletBalance.toFixed(4)} SOL`;

        if (totalLatency > 100) {
            msg += `\n\n⚠️ <b>ALERTE LATENCE ÉLEVÉE</b> : Vérifiez la charge du VPS !`;
            this.warn(`HIGH LATENCY DETECTED: ${totalLatency.toFixed(2)}ms`);
        }

        this.info(`Diagnostic Trade Info for ${tokenName} - Latency: ${totalLatency.toFixed(2)}ms`);
        await this.sendTelegramMsg(msg);
    }

    public async notifySell(tokenName: string, pnlPercentage: number, amountSol: number) {
        const emoji = pnlPercentage >= 0 ? "💰" : "🩸";
        const msg = `${emoji} <b>Sell</b>\nToken: ${tokenName}\nProfit/Loss: ${pnlPercentage.toFixed(2)}%\nMontant: ${amountSol.toFixed(4)} SOL`;
        this.info(msg.replace(/\n/g, " | ").replace(/<[^>]*>?/gm, ''));
        await this.sendTelegramMsg(msg);
    }
}

export const logger = new BotLogger();
