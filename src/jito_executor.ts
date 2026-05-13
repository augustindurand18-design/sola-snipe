import { Connection, Keypair, PublicKey, VersionedTransaction } from "@solana/web3.js";
import { searcherClient } from "jito-ts/dist/sdk/block-engine/searcher";
import { Bundle } from "jito-ts/dist/sdk/block-engine/types";
import bs58 from "bs58";
import { logger } from "./utils/logger.js";
import { Vault } from "./security/vault.js";
import path from "path";
import { fileURLToPath } from "url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

export class JitoExecutor {
    private clients: ReturnType<typeof searcherClient>[] = [];
    private keypair: Keypair;
    private connection: Connection;

    constructor(rpcUrl: string, passphrase: string) {
        const vaultPath = path.join(__dirname, "..", "wallet.enc");
        const privateKeyBase58 = Vault.decrypt(vaultPath, passphrase);
        this.keypair = Keypair.fromSecretKey(bs58.decode(privateKeyBase58));
        this.connection = new Connection(rpcUrl, "processed");

        // Multi-Engine Setup
        const endpoints = [
            "amsterdam.mainnet.block-engine.jito.wtf",
            "frankfurt.mainnet.block-engine.jito.wtf"
        ];

        if (process.env.PAPER_TRADING !== 'true') {
            for (const endpoint of endpoints) {
                this.clients.push(searcherClient(endpoint, this.keypair));
                logger.info(`Initialized Jito searcher client for ${endpoint}`);
            }
        } else {
            logger.warn("PAPER TRADING: Jito Searcher Client connection bypassed (No authentication required).");
        }
        
        logger.info(`Jito Executor ready with decrypted key for ${this.keypair.publicKey.toBase58()}`);
    }

    public getWallet(): Keypair {
        return this.keypair;
    }

    public cachedBalance: number = 0;

    public async checkBalance(minBalanceSol: number): Promise<boolean> {
        const balanceLamports = await this.connection.getBalance(this.keypair.publicKey);
        this.cachedBalance = balanceLamports / 1_000_000_000;
        logger.info(`Current wallet balance: ${this.cachedBalance} SOL`);
        if (this.cachedBalance < minBalanceSol) {
            logger.warn(`Wallet balance (${this.cachedBalance} SOL) is below minimum threshold (${minBalanceSol} SOL).`);
            return false;
        }
        return true;
    }

    private async getDynamicTipLamports(competitionLevel: 'normal' | 'high' = 'normal'): Promise<number> {
        try {
            const response = await fetch("https://bundles.jito.wtf/api/v1/bundles/tip_floor");
            const data = await response.json();
            if (data && data.length > 0) {
                if (competitionLevel === 'high') {
                    // Fetch 90th percentile for aggressive inclusion
                    const highTipSol = data[0].landed_tips_90th_percentile;
                    const dynamicTipLamports = Math.floor(highTipSol * 1_000_000_000);
                    return Math.max(dynamicTipLamports, 100000); // Minimum 0.0001 SOL
                } else {
                    // Fetch 50th percentile (median) and add 10%
                    const medianTipSol = data[0].landed_tips_50th_percentile;
                    const dynamicTipSol = medianTipSol * 1.10;
                    const dynamicTipLamports = Math.floor(dynamicTipSol * 1_000_000_000);
                    return Math.max(dynamicTipLamports, 100000); // Minimum 0.0001 SOL
                }
            }
        } catch (error) {
            logger.warn(`Failed to fetch Jito tip floor. Using default fallback. Error: ${error}`);
        }
        // Fallback tip: 0.001 SOL
        return 1_000_000;
    }

    public async sendBundleWithTip(
        transactions: VersionedTransaction[],
        tipAccountStr: string,
        competitionLevel: 'normal' | 'high' = 'normal'
    ) {
        try {
            // 1. Simulation
            logger.info(`Simulating transaction before sending bundle...`);
            for (const tx of transactions) {
                const simResult = await this.connection.simulateTransaction(tx);
                if (simResult.value.err) {
                    logger.error(`Simulation failed! Aborting Jito bundle to save tip. Error: ${JSON.stringify(simResult.value.err)}`);
                    return;
                }
            }
            logger.info(`Simulation successful.`);

            // 2. Dynamic Tip
            const tipAmountLamports = await this.getDynamicTipLamports(competitionLevel);
            const tipAccount = new PublicKey(tipAccountStr);
            const bundle = new Bundle(transactions, 5); // bundle size 5

            // Usually, we add a tip transaction to the bundle here
            // const tipTx = ...
            // bundle.addTransactions(tipTx)

            logger.info(`Sending bundle via Jito to ${tipAccount.toBase58()} with dynamic tip ${tipAmountLamports} lamports`);
            
            // 3. Multi-Engine Sending
            const sendPromises = this.clients.map(client => client.sendBundle(bundle));
            
            // Wait for at least one to resolve, or all to fail
            const uuid = await Promise.any(sendPromises);
            logger.info(`Bundle sent successfully! UUID: ${uuid}`);
            
        } catch (error) {
            logger.error(`Failed to send Jito bundle: ${error}`);
        }
    }

    public destroy() {
        logger.info("Nettoyage des données sensibles en mémoire (Keypair wipe)...");
        if (this.keypair && this.keypair.secretKey) {
            this.keypair.secretKey.fill(0); // overwrite memory with zeros
        }
        (this.keypair as any) = null;
        if (typeof global.gc === 'function') {
            global.gc();
        }
    }
}
