import { JitoExecutor } from "../jito_executor";
import { logger } from "../utils/logger";

export interface Position {
    tokenAddress: string;
    buyPriceSol: number;
    amountTokens: number;
    highestPriceSeenSol: number;
    hasDerisked: boolean;
    lastLiquiditySol: number;
    isSelling?: boolean;
}

export class ExitManager {
    private positions: Map<string, Position> = new Map();
    private jito: JitoExecutor;
    public onPositionClosed?: (tokenAddress: string) => void;

    constructor(jito: JitoExecutor) {
        this.jito = jito;
    }

    public startMonitoring(
        tokenAddress: string,
        buyPriceSol: number,
        amountTokens: number,
        initialLiquiditySol: number
    ) {
        this.positions.set(tokenAddress, {
            tokenAddress,
            buyPriceSol,
            amountTokens,
            highestPriceSeenSol: buyPriceSol,
            hasDerisked: false,
            lastLiquiditySol: initialLiquiditySol
        });
        logger.info(`Started monitoring position for ${tokenAddress} at ${buyPriceSol.toExponential(4)} SOL. Liquidity: ${initialLiquiditySol} SOL.`);
    }

    public async onPriceUpdate(tokenAddress: string, currentPriceSol: number, currentLiquiditySol: number) {
        const position = this.positions.get(tokenAddress);
        if (!position || position.isSelling) return;

        if (currentPriceSol > position.highestPriceSeenSol) {
            position.highestPriceSeenSol = currentPriceSol;
        }

        const profitPercentage = ((currentPriceSol - position.buyPriceSol) / position.buyPriceSol) * 100;

        // Emergency Exit: Liquidity drops by > 30% in a single update
        const liquidityDropPercentage = ((position.lastLiquiditySol - currentLiquiditySol) / position.lastLiquiditySol) * 100;
        if (liquidityDropPercentage > 30) {
            logger.warn(`EMERGENCY EXIT: Liquidity dropped by ${liquidityDropPercentage.toFixed(2)}% for ${tokenAddress}! Executing market sell.`);
            await this.executeSell(position, 100, "Emergency Exit", profitPercentage);
            return; // Position closed
        }

        position.lastLiquiditySol = currentLiquiditySol; // update for next check

        // --- HIT AND RUN STRATEGY (Ultra-Agressive) ---

        // 1. Hard Stop-Loss (-20%)
        if (profitPercentage <= -20) {
            logger.warn(`HARD STOP-LOSS déclenché à ${profitPercentage.toFixed(2)}% pour ${tokenAddress}. Liquidation immédiate.`);
            position.isSelling = true;
            await this.executeSell(position, 100, "Hard Stop-Loss", profitPercentage);
            return;
        }

        // 2. Take-Profit Agressif (+30%)
        if (profitPercentage >= 30) {
            logger.info(`🎯 TAKE-PROFIT ATTEINT (+${profitPercentage.toFixed(2)}%) pour ${tokenAddress}. Vente totale.`);
            position.isSelling = true;
            await this.executeSell(position, 100, "Take-Profit HFT", profitPercentage);
            return;
        }

        // 3. Trailing Stop-Loss de sécurité si ça stagne après un petit profit
        const highestProfitPercentage = ((position.highestPriceSeenSol - position.buyPriceSol) / position.buyPriceSol) * 100;
        if (highestProfitPercentage >= 15 && profitPercentage <= 5) {
            logger.info(`Trailing Stop-Loss déclenché (Gains effacés) pour ${tokenAddress}. Sortie à +${profitPercentage.toFixed(2)}%.`);
            position.isSelling = true;
            await this.executeSell(position, 100, "Trailing Stop-Loss", profitPercentage);
            return;
        }
    }

    private async executeSell(position: Position, percentage: number, reason: string, pnlPercentage: number) {
        logger.info(`[${reason}] Constructing Jito bundle to sell ${percentage}% of ${position.tokenAddress}...`);

        // Placeholder for the actual transaction construction
        const dummyTx = [] as any;

        const tipAccount = "random_tip_account"; // Placeholder

        // await this.jito.sendBundleWithTip(dummyTx, tipAccount);

        // Calculate amount sol for display (mock logic)
        const amountSol = (position.amountTokens * (percentage / 100)) * position.buyPriceSol * (1 + pnlPercentage / 100);
        await logger.notifySell(position.tokenAddress, pnlPercentage, amountSol);

        if (percentage === 100) {
            this.positions.delete(position.tokenAddress);
            logger.info(`Position closed for ${position.tokenAddress}.`);
            if (this.onPositionClosed) {
                this.onPositionClosed(position.tokenAddress);
            }
        } else {
            position.amountTokens -= position.amountTokens * (percentage / 100);
            logger.info(`Position reduced for ${position.tokenAddress}. Remaining: ${position.amountTokens}`);
        }
    }
}
