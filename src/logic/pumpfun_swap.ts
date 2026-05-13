import { PublicKey, TransactionInstruction, SystemProgram, SYSVAR_RENT_PUBKEY } from "@solana/web3.js";
// We use a manual ATA calculation to avoid @solana/spl-token dependency if it's missing, but let's assume it's here:
// Actually, to be safe and fastest, we implement ATA derivation inline:
export const SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID = new PublicKey('ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL');
export const TOKEN_PROGRAM_ID = new PublicKey('TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA');

export function getAssociatedTokenAddressSync(mint: PublicKey, owner: PublicKey): PublicKey {
    const [address] = PublicKey.findProgramAddressSync(
        [owner.toBuffer(), TOKEN_PROGRAM_ID.toBuffer(), mint.toBuffer()],
        SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID
    );
    return address;
}

export class PumpfunSwap {
    public static readonly PROGRAM_ID = new PublicKey("6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P");
    public static readonly GLOBAL_ACCOUNT = new PublicKey("4wTV1YmiEkRvAtNtsSGPtUrqRYQMe5SKy2uB4Jjaxnjf");
    public static readonly FEE_RECIPIENT = new PublicKey("CebN5WGQ4jvEPvsVU4EoHEpgzq1VV7AbicfhtW4xC9iM");
    public static readonly EVENT_AUTHORITY = new PublicKey("Ce6TQqeHC9p8KetsN6JsjHK7UTZkPNrwXXz4peE32PEp");

    public static buildCreateATAInstruction(
        payer: PublicKey,
        owner: PublicKey,
        mint: PublicKey
    ): TransactionInstruction {
        const ata = getAssociatedTokenAddressSync(mint, owner);
        return new TransactionInstruction({
            programId: SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID,
            keys: [
                { pubkey: payer, isSigner: true, isWritable: true },
                { pubkey: ata, isSigner: false, isWritable: true },
                { pubkey: owner, isSigner: false, isWritable: false },
                { pubkey: mint, isSigner: false, isWritable: false },
                { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
                { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
            ],
            data: Buffer.from([1]) // 1 = CreateIdempotent (does not fail if account already exists)
        });
    }

    public static getTokensOut(
        solAmount: number,
        virtualSolReserves: number,
        virtualTokenReserves: number
    ): bigint {
        const k = BigInt(virtualSolReserves) * BigInt(virtualTokenReserves);
        const newVirtualSolReserves = BigInt(virtualSolReserves) + BigInt(Math.floor(solAmount * 1e9));
        const newVirtualTokenReserves = k / newVirtualSolReserves;
        const tokensOut = BigInt(virtualTokenReserves) - newVirtualTokenReserves;
        return tokensOut;
    }

    /**
     * Builds the instruction to BUY a token on Pump.fun
     */
    public static buildBuyInstruction(
        buyer: PublicKey,
        mint: PublicKey,
        amountTokens: bigint,
        maxSolCost: bigint
    ): TransactionInstruction {
        const [bondingCurve] = PublicKey.findProgramAddressSync(
            [Buffer.from("bonding-curve"), mint.toBuffer()],
            this.PROGRAM_ID
        );

        const associatedBondingCurve = getAssociatedTokenAddressSync(mint, bondingCurve);
        const associatedUser = getAssociatedTokenAddressSync(mint, buyer);

        const keys = [
            { pubkey: this.GLOBAL_ACCOUNT, isSigner: false, isWritable: false },
            { pubkey: this.FEE_RECIPIENT, isSigner: false, isWritable: true },
            { pubkey: mint, isSigner: false, isWritable: false },
            { pubkey: bondingCurve, isSigner: false, isWritable: true },
            { pubkey: associatedBondingCurve, isSigner: false, isWritable: true },
            { pubkey: associatedUser, isSigner: false, isWritable: true },
            { pubkey: buyer, isSigner: true, isWritable: true },
            { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
            { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
            { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false },
            { pubkey: this.EVENT_AUTHORITY, isSigner: false, isWritable: false },
            { pubkey: this.PROGRAM_ID, isSigner: false, isWritable: false },
        ];

        // Discriminator for "buy" (8 bytes) + amount (8 bytes) + max_sol_cost (8 bytes)
        const data = Buffer.alloc(24);
        data.set(Buffer.from([102, 6, 61, 18, 1, 218, 235, 234]), 0);
        data.writeBigUInt64LE(amountTokens, 8);
        data.writeBigUInt64LE(maxSolCost, 16);

        return new TransactionInstruction({
            programId: this.PROGRAM_ID,
            keys,
            data
        });
    }

    /**
     * Builds the instruction to SELL a token on Pump.fun
     */
    public static buildSellInstruction(
        seller: PublicKey,
        mint: PublicKey,
        amountTokens: bigint,
        minSolOutput: bigint
    ): TransactionInstruction {
        const [bondingCurve] = PublicKey.findProgramAddressSync(
            [Buffer.from("bonding-curve"), mint.toBuffer()],
            this.PROGRAM_ID
        );

        const associatedBondingCurve = getAssociatedTokenAddressSync(mint, bondingCurve);
        const associatedUser = getAssociatedTokenAddressSync(mint, seller);

        const keys = [
            { pubkey: this.GLOBAL_ACCOUNT, isSigner: false, isWritable: false },
            { pubkey: this.FEE_RECIPIENT, isSigner: false, isWritable: true },
            { pubkey: mint, isSigner: false, isWritable: false },
            { pubkey: bondingCurve, isSigner: false, isWritable: true },
            { pubkey: associatedBondingCurve, isSigner: false, isWritable: true },
            { pubkey: associatedUser, isSigner: false, isWritable: true },
            { pubkey: seller, isSigner: true, isWritable: true },
            { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
            { pubkey: SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID, isSigner: false, isWritable: false },
            { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
            { pubkey: this.EVENT_AUTHORITY, isSigner: false, isWritable: false },
            { pubkey: this.PROGRAM_ID, isSigner: false, isWritable: false },
        ];

        // Discriminator for "sell" (8 bytes) + amount (8 bytes) + min_sol_output (8 bytes)
        const data = Buffer.alloc(24);
        data.set(Buffer.from([51, 230, 250, 225, 87, 196, 11, 252]), 0);
        data.writeBigUInt64LE(amountTokens, 8);
        data.writeBigUInt64LE(minSolOutput, 16);

        return new TransactionInstruction({
            programId: this.PROGRAM_ID,
            keys,
            data
        });
    }
}
