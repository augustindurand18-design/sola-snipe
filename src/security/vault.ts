import crypto from "crypto";
import fs from "fs";

const ALGORITHM = "aes-256-gcm";

export class Vault {
    public static encrypt(privateKeyBase58: string, passphrase: string): Buffer {
        // Derive key from passphrase
        const key = crypto.scryptSync(passphrase, "salt", 32);
        const iv = crypto.randomBytes(12); // 96-bit IV
        
        const cipher = crypto.createCipheriv(ALGORITHM, key, iv);
        
        const encrypted = Buffer.concat([
            cipher.update(privateKeyBase58, "utf8"),
            cipher.final()
        ]);
        
        const authTag = cipher.getAuthTag();
        
        // Return packed buffer: iv(12) + authTag(16) + encryptedData
        return Buffer.concat([iv, authTag, encrypted]);
    }

    public static decrypt(encryptedVaultPath: string, passphrase: string): string {
        if (!fs.existsSync(encryptedVaultPath)) {
            throw new Error(`Vault file not found at ${encryptedVaultPath}`);
        }
        
        const fileBuffer = fs.readFileSync(encryptedVaultPath);
        
        const iv = fileBuffer.subarray(0, 12);
        const authTag = fileBuffer.subarray(12, 28);
        const encryptedData = fileBuffer.subarray(28);
        
        const key = crypto.scryptSync(passphrase, "salt", 32);
        
        const decipher = crypto.createDecipheriv(ALGORITHM, key, iv);
        decipher.setAuthTag(authTag);
        
        const decrypted = Buffer.concat([
            decipher.update(encryptedData),
            decipher.final()
        ]);
        
        return decrypted.toString("utf8");
    }
}
