import { Connection, PublicKey } from "@solana/web3.js";
const connection = new Connection("https://mainnet.helius-rpc.com/?api-key=e03668c9-f984-45c7-ac22-e0ad3a9695e4", {
    wsEndpoint: "wss://mainnet.helius-rpc.com/?api-key=e03668c9-f984-45c7-ac22-e0ad3a9695e4"
});
const PUMP_FUN = new PublicKey("6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P");

connection.onLogs(PUMP_FUN, async (logs) => {
    if (logs.err) return;
    
    if (logs.logs.some(l => l.includes("Instruction: CreateV2"))) {
        console.log("Found CreateV2!", logs.signature);
        const parsedTx = await connection.getParsedTransaction(logs.signature, {
            maxSupportedTransactionVersion: 0,
            commitment: "confirmed"
        });
        
        if (!parsedTx || !parsedTx.transaction) {
            console.log("No parsedTx");
            return;
        }
        
        for (const ix of parsedTx.transaction.message.instructions) {
            if ('programId' in ix && ix.programId.toBase58() === PUMP_FUN.toBase58()) {
                console.log("PumpFun Instruction:");
                if ('accounts' in ix) {
                    console.log("Accounts length:", ix.accounts.length);
                    console.log("First 3 accounts:");
                    for (let i=0; i<Math.min(3, ix.accounts.length); i++) {
                        console.log(`[${i}]`, ix.accounts[i].toBase58());
                    }
                    console.log("Data length:", ix.data.length);
                } else {
                    console.log("No accounts field! It was fully parsed?");
                    console.log(ix);
                }
            }
        }
        process.exit(0);
    }
}, "processed");
console.log("Listening for CreateV2...");
