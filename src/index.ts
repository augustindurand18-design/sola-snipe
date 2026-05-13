import dotenv from "dotenv";
import readline from "readline";
import { logger } from "./utils/logger.js";
import { GeyserClient } from "./geyser.js";
import { RugChecker } from "./rug_check.js";
import { JitoExecutor } from "./jito_executor.js";
import { TradeOptimizer } from "./logic/trade_optimizer.js";
import { ExitManager } from "./logic/exit_manager.js";
import { PriceFeed } from "./logic/price_feed.js";
import { PumpfunSwap } from "./logic/pumpfun_swap.js";
import { PumpFunParser } from "./parser/pumpfun_parser.js";
import { Connection, PublicKey, TransactionMessage, VersionedTransaction } from "@solana/web3.js";

dotenv.config();

// Mainnet Pump.fun Program ID
const PUMP_FUN_PROGRAM_ID = "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P";
// Raydium Liquidity Pool V4
const RAYDIUM_V4_PROGRAM_ID = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8";

const MIN_LIQUIDITY_SOL = 15; // Configuration

function askPassphrase(): Promise<string> {
    return new Promise((resolve) => {
        const rl = readline.createInterface({
            input: process.stdin,
            output: process.stdout
        });
        rl.question("Vault Passphrase : ", (answer) => {
            rl.close();
            resolve(answer);
        });
    });
}

async function main() {
    logger.info("Starting Solana HFT Meme Coin Sniper...");

    const geyserUrl = process.env.GEYSER_RPC_URL;
    const geyserToken = process.env.GEYSER_API_TOKEN;
    const rpcUrl = process.env.RPC_URL || "https://api.mainnet-beta.solana.com";

    const PAPER_TRADING = process.env.PAPER_TRADING === 'true';
    const COMPETITION_LEVEL = process.env.COMPETITION_LEVEL === 'high' ? 'high' : 'normal';
    const MIN_BALANCE_THRESHOLD = 0.1; // Configuration: 0.1 SOL minimum

    if (!geyserUrl) {
        logger.error("Missing required environment variables.");
        process.exit(1);
    }

    const passphrase = await askPassphrase();

    let jito: JitoExecutor;
    try {
        jito = new JitoExecutor(rpcUrl, passphrase);
    } catch (e: any) {
        logger.error(`Failed to decrypt vault: ${e.message}`);
        process.exit(1);
    }

    const hasBalance = await jito.checkBalance(MIN_BALANCE_THRESHOLD);
    if (!hasBalance && !PAPER_TRADING) {
        logger.error("Insufficient balance for real trading. Aborting startup.");
        process.exit(1);
    }

    if (PAPER_TRADING) {
        logger.info("PAPER TRADING MODE IS ACTIVE. No real funds will be used.");
    }

    // Initialize logic managers
    const exitManager = new ExitManager(jito);
    const geyser = new GeyserClient(geyserUrl, geyserToken);
    const rpcConnection = geyser.getConnection() || new Connection(rpcUrl);
    const priceFeed = new PriceFeed(rpcConnection, exitManager);
    
    exitManager.onPositionClosed = (token: string) => {
        priceFeed.unsubscribe(token);
    };

    await geyser.subscribe(PUMP_FUN_PROGRAM_ID, async (tx) => {
        const detectionStartTime = performance.now();
        
        // 1. Extract true data via Parser
        const poolData = await PumpFunParser.parse(tx, geyser.getConnection());
        if (!poolData) return; // Silently ignore all non-creation spam

        const initialLiquiditySol = poolData.initialLiquiditySol;
        const isLpBurned = true; // Pump.fun is rug-proof initially (no LP to rug)
        const tokenAddress = poolData.mint;

        // --- DEV BUY FILTER (HFT GUERILLA MODE) ---
        if (poolData.devBuySol < 0.01) {
            logger.info(`Ignoré: ${tokenAddress} - Le dev n'a acheté que ${poolData.devBuySol.toFixed(2)} SOL. (Création à blanc sans investissement)`);
            return;
        }

        await logger.notifyDetection(`https://pump.fun/${tokenAddress} | DEV BUY: ${poolData.devBuySol.toFixed(2)} SOL`, initialLiquiditySol);

        const isSafe = RugChecker.isRugProof(tx);
        
        let clusterAnalysis = { isRug: false, reason: "" };
        if (isSafe) {
            clusterAnalysis = await RugChecker.analyzeWalletClusters(tokenAddress);
        }

        const t1End = performance.now();
        const t1Ms = t1End - detectionStartTime;

        if (isSafe && !clusterAnalysis.isRug) {
            const killSwitch = TradeOptimizer.shouldKillSwitch(initialLiquiditySol, MIN_LIQUIDITY_SOL, isLpBurned);
            if (killSwitch) {
                logger.warn("Kill switch triggered. Aborting Jito bundle.");
                return;
            }

            const slippage = TradeOptimizer.calculateDynamicSlippage(initialLiquiditySol);
            logger.info(`Proceeding with swap. Dynamic slippage set to: ${slippage * 100}%`);

            // 3. Construct buy transaction and send via Jito Bundle
            // const buyTx = constructBuyTransaction(...);
            if (PAPER_TRADING) {
                logger.info(`ACHAT SIMULÉ : Simulation de l'envoi du bundle Jito pour ${tokenAddress} (Competition: ${COMPETITION_LEVEL})`);
            } else {
                logger.info(`ACHAT RÉEL : Envoi du bundle Jito pour ${tokenAddress} (Competition: ${COMPETITION_LEVEL})`);
                // const uuid = await jito.sendBundleWithTip([buyTx], "random_tip_account", COMPETITION_LEVEL);
            }
            
            const t2End = performance.now();
            const t2Ms = t2End - t1End;

            // Pump.fun Bonding Curve Constants
            const virtualSolReserves = 30_000_000_000; // 30 SOL
            const virtualTokenReserves = 1_073_000_000_000_000; // 1.073 Billion Tokens
            
            const vToken = virtualTokenReserves / 1e6;
            const vSol = virtualSolReserves / 1e9;
            const buyPriceSol = vSol / vToken;

            try {
                // --- TRANSACTION FORGING ---
                const wallet = jito.getWallet();
                const mintPubkey = new PublicKey(poolData.mint);
                
                // Montant d'investissement via fichier .env (défaut à 0.05 SOL)
                const solToSpend = parseFloat(process.env.INVESTMENT_SOL || "0.05");
                const exactTokensOut = PumpfunSwap.getTokensOut(solToSpend, virtualSolReserves, virtualTokenReserves);
                
                // Marge de glissement (Slippage dynamique du TradeOptimizer)
                const slippageMultiplier = BigInt(Math.floor((1 - slippage) * 100)); // 1 - 0.20 = 0.80 -> 80
                const minTokensOut = (exactTokensOut * slippageMultiplier) / 100n;
                
                const amountTokens = Number(minTokensOut) / 1e6;

                // 1. ATA Instruction
                const ataIx = PumpfunSwap.buildCreateATAInstruction(wallet.publicKey, wallet.publicKey, mintPubkey);
                
                // 2. Buy Instruction
                const maxSolCost = BigInt(Math.floor(solToSpend * 1.5 * 1e9)); // Max 50% de plus au cas où le prix bouge
                const buyIx = PumpfunSwap.buildBuyInstruction(wallet.publicKey, mintPubkey, minTokensOut, maxSolCost);
                
                // 3. Compile Transaction
                const blockhash = await rpcConnection.getLatestBlockhash("processed");
                const messageV0 = new TransactionMessage({
                    payerKey: wallet.publicKey,
                    recentBlockhash: blockhash.blockhash,
                    instructions: [ataIx, buyIx]
                }).compileToV0Message();
                
                const tx = new VersionedTransaction(messageV0);
                tx.sign([wallet]);
                
                logger.info(`Forged Real VersionedTransaction! Size: ${tx.serialize().length} bytes`);
                
                if (PAPER_TRADING) {
                    logger.info(`Simulation de la transaction binaire sur le réseau Solana...`);
                    const sim = await rpcConnection.simulateTransaction(tx);
                    if (sim.value.err) {
                        logger.error(`Simulation failed: ${JSON.stringify(sim.value.err)}`);
                    } else {
                        logger.info(`Simulation Success! ✅ Units consumed: ${sim.value.unitsConsumed}`);
                    }
                } else {
                    logger.info(`ACHAT RÉEL : Envoi du bundle Jito pour ${tokenAddress}`);
                    // jito.sendBundleWithTip([tx], ...);
                }

                const walletBalance = jito.cachedBalance;
                await logger.notifyBuy(tokenAddress, t1Ms, t2Ms, slippage * 100, walletBalance);

                // 4. Start monitoring the position for exits
                exitManager.startMonitoring(tokenAddress, buyPriceSol, amountTokens, initialLiquiditySol);
                priceFeed.subscribeToBondingCurve(tokenAddress, poolData.bondingCurve);

            } catch (e) {
                logger.warn(`Transaction build failed: ${e}`);
            }
            // Example of how price updates would trigger exits (this would usually run in a separate WebSocket listener)
            // setTimeout(() => exitManager.onPriceUpdate(tokenAddress, 0.00002, 25), 5000); // 100% profit (De-risking)
        }
    });

    // Optionally listen to Raydium as well
    // await geyser.subscribe(RAYDIUM_V4_PROGRAM_ID, async (tx) => { ... });

    // --- Graceful Shutdown & Memory Wipe ---
    let isShuttingDown = false;
    const shutdown = async (signal: string) => {
        if (isShuttingDown) return;
        isShuttingDown = true;
        logger.warn(`Received ${signal}. Shutting down gracefully...`);
        
        try {
            if (geyser) geyser.close();
            if (jito) jito.destroy();
            logger.info("Arrêt terminé. Au revoir.");
            process.exit(0);
        } catch (e) {
            logger.error(`Erreur durant l'arrêt: ${e}`);
            process.exit(1);
        }
    };

    process.on('SIGINT', () => shutdown('SIGINT'));
    process.on('SIGTERM', () => shutdown('SIGTERM'));

    process.on('uncaughtException', (err) => {
        logger.error(`Uncaught Exception: ${err.message}`);
        logger.error(err.stack || "");
        shutdown('uncaughtException');
    });
}

main().catch(err => {
    logger.error(`Fatal error: ${err}`);
});
