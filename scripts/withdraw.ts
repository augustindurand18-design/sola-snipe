import { Connection, Keypair, SystemProgram, Transaction, PublicKey, sendAndConfirmTransaction } from "@solana/web3.js";
import { Vault } from "../src/security/vault.js";
import bs58 from "bs58";
import readline from "readline";
import dotenv from "dotenv";
import path from "path";
import { fileURLToPath } from "url";

dotenv.config();

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout
});

const question = (query: string): Promise<string> => {
    return new Promise((resolve) => rl.question(query, resolve));
};

async function main() {
    console.log("=== SOLANA BOT WITHDRAWAL TOOL ===");

    // 1. Get Passphrase & Decrypt
    const passphrase = await question("Veuillez entrer le mot de passe de votre Vault : ");
    const vaultPath = path.join(__dirname, "..", "wallet.enc");
    
    let privateKeyBase58: string;
    try {
        privateKeyBase58 = Vault.decrypt(vaultPath, passphrase);
        console.log("✅ Coffre-fort déverrouillé avec succès.\n");
    } catch (e) {
        console.error("❌ Erreur : Mot de passe incorrect ou coffre-fort corrompu.");
        process.exit(1);
    }

    const keypair = Keypair.fromSecretKey(bs58.decode(privateKeyBase58));
    console.log(`Adresse du Bot : ${keypair.publicKey.toBase58()}`);

    // 2. Check Balance
    const rpcUrl = process.env.RPC_URL || "https://api.mainnet-beta.solana.com";
    const connection = new Connection(rpcUrl, "confirmed");
    const balanceLamports = await connection.getBalance(keypair.publicKey);
    const balanceSol = balanceLamports / 1e9;

    console.log(`Solde actuel : ${balanceSol} SOL\n`);

    if (balanceSol <= 0.001) {
        console.log("Solde insuffisant pour effectuer un retrait.");
        process.exit(0);
    }

    // 3. Get Destination Address
    const destination = await question("Entrez l'adresse Solana de destination (ex: votre Phantom wallet) : ");
    let destPubkey: PublicKey;
    try {
        destPubkey = new PublicKey(destination);
    } catch (e) {
        console.error("❌ Erreur : Adresse Solana invalide.");
        process.exit(1);
    }

    // 4. Calculate amount
    console.log("Rappel : Laissez ~0.001 SOL pour payer les frais de transaction de ce transfert.");
    const amountStr = await question(`Entrez le montant à retirer en SOL (Max conseillé: ${(balanceSol - 0.001).toFixed(4)}) ou tapez 'ALL' : `);
    
    let withdrawLamports = 0;
    if (amountStr.toUpperCase() === "ALL") {
        withdrawLamports = balanceLamports - 100000; // Leave 0.0001 SOL for the tx fee
    } else {
        const amountSol = parseFloat(amountStr);
        if (isNaN(amountSol) || amountSol <= 0 || amountSol > balanceSol) {
            console.error("❌ Erreur : Montant invalide.");
            process.exit(1);
        }
        withdrawLamports = Math.floor(amountSol * 1e9);
    }

    // 5. Send Transaction
    console.log(`\n⏳ Création de la transaction de ${withdrawLamports / 1e9} SOL vers ${destPubkey.toBase58()}...`);
    
    try {
        const tx = new Transaction().add(
            SystemProgram.transfer({
                fromPubkey: keypair.publicKey,
                toPubkey: destPubkey,
                lamports: withdrawLamports,
            })
        );

        const signature = await sendAndConfirmTransaction(connection, tx, [keypair]);
        console.log(`✅ Transfert réussi !`);
        console.log(`🔗 Voir sur Solscan : https://solscan.io/tx/${signature}`);
    } catch (e) {
        console.error(`❌ Échec du transfert : ${e}`);
    }

    // Secure memory wipe
    keypair.secretKey.fill(0);
    rl.close();
}

main().catch(console.error);
