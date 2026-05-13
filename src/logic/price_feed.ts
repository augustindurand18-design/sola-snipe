import { Connection, PublicKey } from "@solana/web3.js";
import { ExitManager } from "./exit_manager.js";
import { logger } from "../utils/logger.js";

export class PriceFeed {
    private connection: Connection;
    private exitManager: ExitManager;
    private subscriptionIds: Map<string, number> = new Map();

    constructor(connection: Connection, exitManager: ExitManager) {
        this.connection = connection;
        this.exitManager = exitManager;
    }

    public subscribeToBondingCurve(mint: string, bondingCurve: string) {
        if (this.subscriptionIds.has(mint)) return;

        logger.info(`🔌 Price Feed connecté sur la Bonding Curve: ${bondingCurve}`);
        const bondingCurvePubKey = new PublicKey(bondingCurve);

        const subId = this.connection.onAccountChange(
            bondingCurvePubKey,
            (accountInfo, context) => {
                const data = accountInfo.data;
                // Bonding curve state must be at least 41 bytes long (8 discriminator + 4x 8 bytes u64 + 1 bool)
                if (data.length < 40) return;

                // Parse u64 (little-endian)
                // virtualTokenReserves starts at offset 8
                const virtualTokenReserves = data.readBigUInt64LE(8);
                // virtualSolReserves starts at offset 16
                const virtualSolReserves = data.readBigUInt64LE(16);

                // Pump.fun decimals: Token = 6, SOL = 9
                const solReserves = Number(virtualSolReserves) / 1e9;
                const tokenReserves = Number(virtualTokenReserves) / 1e6;

                if (tokenReserves === 0) return;

                const currentPriceSol = solReserves / tokenReserves;

                // Trigger ExitManager evaluation
                this.exitManager.onPriceUpdate(mint, currentPriceSol, solReserves);
            },
            "processed"
        );

        this.subscriptionIds.set(mint, subId);
    }

    public unsubscribe(mint: string) {
        const subId = this.subscriptionIds.get(mint);
        if (subId !== undefined) {
            this.connection.removeAccountChangeListener(subId);
            this.subscriptionIds.delete(mint);
            logger.info(`🔌 Price Feed déconnecté pour ${mint}`);
        }
    }
}
