import { logger } from "../utils/logger";

export class TradeOptimizer {
    public static calculateDynamicSlippage(initialLiquiditySol: number): number {
        if (initialLiquiditySol < 20) {
            return 0.40; // 40%
        } else if (initialLiquiditySol >= 20 && initialLiquiditySol < 50) {
            return 0.20; // 20%
        } else {
            return 0.10; // 10%
        }
    }

    public static shouldKillSwitch(
        initialLiquiditySol: number,
        minLiquiditySol: number,
        isLpBurned: boolean
    ): boolean {
        if (initialLiquiditySol < minLiquiditySol) {
            logger.warn(`Kill Switch Triggered: Initial liquidity (${initialLiquiditySol} SOL) is below minimum (${minLiquiditySol} SOL).`);
            return true;
        }

        if (!isLpBurned) {
            logger.warn(`Kill Switch Triggered: LP Burn ratio not detected in creation instruction.`);
            return true;
        }

        return false;
    }
}
