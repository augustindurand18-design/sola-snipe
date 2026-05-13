import { Vault } from "../src/security/vault.js";
import { Keypair } from "@solana/web3.js";
import bs58 from "bs58";
import readline from "readline";
import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout
});

rl.question("Avez-vous déjà une clé privée Solana ? (O/N) : ", (answer) => {
    if (answer.toLowerCase() === 'n') {
        const newKeypair = Keypair.generate();
        const privKey = bs58.encode(newKeypair.secretKey);
        const pubKey = newKeypair.publicKey.toBase58();
        console.log(`\n✅ NOUVEAU WALLET GÉNÉRÉ !`);
        console.log(`Adresse Publique : ${pubKey}`);
        console.log(`(Gardez cette adresse pour envoyer des fonds au bot)\n`);
        
        rl.question("Entrez la passphrase de chiffrement de votre choix (mot de passe) : ", (passphrase) => {
            const encrypted = Vault.encrypt(privKey, passphrase);
            const outputPath = path.join(__dirname, "..", "wallet.enc");
            fs.writeFileSync(outputPath, encrypted);
            console.log(`✅ Clé privée chiffrée et sauvegardée dans : ${outputPath}`);
            rl.close();
        });
    } else {
        rl.question("\nEntrez votre clé privée Solana (Format Base58) : ", (privateKey) => {
            rl.question("Entrez la passphrase de chiffrement (mot de passe) : ", (passphrase) => {
                const encrypted = Vault.encrypt(privateKey, passphrase);
                const outputPath = path.join(__dirname, "..", "wallet.enc");
                fs.writeFileSync(outputPath, encrypted);
                console.log(`✅ Clé privée chiffrée et sauvegardée dans : ${outputPath}`);
                rl.close();
            });
        });
    }
});
