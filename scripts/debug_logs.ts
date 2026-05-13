import { Connection, PublicKey } from "@solana/web3.js";
const connection = new Connection("https://mainnet.helius-rpc.com/?api-key=e03668c9-f984-45c7-ac22-e0ad3a9695e4", {
    wsEndpoint: "wss://mainnet.helius-rpc.com/?api-key=e03668c9-f984-45c7-ac22-e0ad3a9695e4"
});
const PUMP_FUN = new PublicKey("6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P");

const seen = new Set<string>();

connection.onLogs(PUMP_FUN, (logs) => {
    if (logs.err) return;
    
    let isPump = false;
    for (const log of logs.logs) {
        if (log.includes("6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P invoke")) {
            isPump = true;
        } else if (isPump && log.includes("Program log: Instruction: ")) {
            const ixName = log.split("Instruction: ")[1];
            if (!seen.has(ixName)) {
                seen.add(ixName);
                console.log("New Pump.fun Instruction seen:", ixName);
            }
            isPump = false;
        } else if (log.includes("invoke")) {
            isPump = false;
        }
    }
}, "processed");
console.log("Listening for unique Pump.fun instructions...");
