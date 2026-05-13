import { Vault } from "../src/security/vault.js";
import fs from "fs";
import { Keypair } from "@solana/web3.js";
import bs58 from "bs58";
import path from "path";
import { fileURLToPath } from "url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const keypair = Keypair.generate();
const privateKey = bs58.encode(keypair.secretKey);
const passphrase = "test";

const encrypted = Vault.encrypt(privateKey, passphrase);
const outputPath = path.join(__dirname, "..", "wallet.enc");
fs.writeFileSync(outputPath, encrypted);
console.log("Dummy wallet created.");
