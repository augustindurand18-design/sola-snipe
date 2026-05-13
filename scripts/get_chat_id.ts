import dotenv from "dotenv";
dotenv.config();
import fs from "fs";

const token = process.env.TELEGRAM_BOT_TOKEN;
if (!token) {
    console.error("Token non trouvé dans .env");
    process.exit(1);
}

async function check() {
    console.log("En attente d'un message sur votre bot Telegram...");
    try {
        const response = await fetch(`https://api.telegram.org/bot${token}/getUpdates`);
        const data = await response.json();
        
        if (data.ok && data.result.length > 0) {
            const chatId = data.result[data.result.length - 1].message?.chat?.id;
            if (!chatId) {
                setTimeout(check, 3000);
                return;
            }
            console.log(`\n\n🎉 CHAT ID TROUVÉ : ${chatId}`);
            
            let envData = fs.readFileSync(".env", "utf8");
            envData = envData.replace(/TELEGRAM_CHAT_ID=.*/, `TELEGRAM_CHAT_ID=${chatId}`);
            fs.writeFileSync(".env", envData);
            console.log("Le fichier .env a été mis à jour automatiquement !");
            console.log("Vous pouvez maintenant lancer le bot avec: echo test | npx tsx src/index.ts");
            process.exit(0);
        } else {
            setTimeout(check, 3000); // Check again in 3s
        }
    } catch (e) {
        console.error("Erreur:", e);
        setTimeout(check, 3000);
    }
}

check();
