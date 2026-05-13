import { Connection } from "@solana/web3.js";
import type { ParsedTransactionWithMeta } from "@solana/web3.js";
import { logger } from "../utils/logger.js";

export interface ParsedPool {
    mint: string;
    bondingCurve: string;
    creator: string;
    initialLiquiditySol: number;
    devBuySol: number;
}

export class PumpFunParser {
    // Pump.fun Program ID
    public static readonly PROGRAM_ID = "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P";

    /**
     * Parse either a WSS log array or a gRPC binary transaction to extract Pool details.
     */
    public static async parse(tx: any, connection?: Connection): Promise<ParsedPool | null> {
        // 1. WSS Mock Mode
        if (tx.isWssMock) {
            const logs: string[] = tx.logs || [];

            // Fast filter: Does it even contain the Create instruction?
            const isCreation = logs.some(log => log.includes("Instruction: CreateV2"));
            if (!isCreation) {
                return null; // Ignore spam
            }

            if (!connection) {
                logger.error("Connection object required for WSS fallback parsing.");
                return null;
            }

            logger.info(`[FREE WSS MODE] Creation detected! Waiting for node confirmation to extract data...`);

            // Fetch parsed transaction with retry loop (since 'processed' txs aren't immediately available via RPC)
            try {
                let parsedTx = null;
                for (let i = 0; i < 5; i++) {
                    // Wait 400ms between attempts
                    await new Promise(r => setTimeout(r, 400));

                    parsedTx = await connection.getParsedTransaction(tx.signature, {
                        maxSupportedTransactionVersion: 0,
                        commitment: "confirmed"
                    });

                    if (parsedTx && parsedTx.transaction && parsedTx.transaction.message) {
                        break;
                    }
                }

                if (!parsedTx || !parsedTx.transaction || !parsedTx.transaction.message) {
                    logger.warn(`Could not fetch transaction details for ${tx.signature} after 2 seconds.`);
                    return null;
                }

                return this.extractFromParsedTx(parsedTx);
            } catch (e) {
                logger.error(`Error fetching/parsing WSS transaction ${tx.signature}: ${e}`);
                return null;
            }
        }

        // 2. gRPC Mode (Binary Buffer)
        // In a real gRPC stream, tx contains the full parsed message
        // For now, we return a mock parsed pool if it matches gRPC logic
        // We will implement full gRPC binary parsing when required
        // ...
        return null;
    }

    private static extractFromParsedTx(parsedTx: ParsedTransactionWithMeta): ParsedPool | null {
        const accountKeys = parsedTx.transaction.message.accountKeys;
        let mint = "";
        let bondingCurve = "";
        let creator = "";

        // Pump.fun Create Instruction Accounts mapping (based on IDL):
        // 0. mint
        // 1. mintAuthority
        // 2. bondingCurve
        // 3. associatedBondingCurve
        // 4. global
        // 5. mplTokenMetadata
        // 6. metadata
        // 7. user (creator)
        // ...

        // We find the instruction for Pump.fun
        const instructions = parsedTx.transaction.message.instructions;
        for (const ix of instructions) {
            if (!('programId' in ix)) continue; // skip fully decoded? Actually both have programId.

            if (ix.programId.toBase58() === this.PROGRAM_ID) {
                // If it's a partially decoded instruction it has an 'accounts' array
                if ('accounts' in ix && ix.accounts.length >= 8) {
                    mint = ix.accounts[0].toBase58();
                    bondingCurve = ix.accounts[2].toBase58();
                    creator = ix.accounts[7].toBase58();

                    let devBuySol = 0;

                    // Calculate Dev Buy by looking at their SOL balance change
                    const meta = parsedTx.meta;
                    const message = parsedTx.transaction.message;
                    if (meta && meta.preBalances && meta.postBalances) {
                        const accountKeys = message.accountKeys as any[];
                        // In PartiallyDecoded or Parsed format, accountKeys might be an array of objects
                        const curveIndex = accountKeys.findIndex((k: any) =>
                            (k.pubkey ? k.pubkey.toBase58() : k.toBase58?.() || k) === bondingCurve
                        );

                        if (curveIndex !== -1) {
                            const postBalance = meta.postBalances[curveIndex];
                            // Le solde final contient l'exemption de loyer (0.00239424 SOL) + le total des achats !
                            const injectedSol = (postBalance / 1e9) - 0.00239424;

                            if (injectedSol > 0.001) {
                                devBuySol = injectedSol; // Represents ALL bundled buys in the creation block!
                            }
                        }
                    }

                    return {
                        mint,
                        bondingCurve,
                        creator,
                        initialLiquiditySol: 30, // Pump.fun standard virtual curve start
                        devBuySol
                    };
                }
            }
        }

        return null;
    }
}
