// import Client from "@triton-one/yellowstone-grpc";
import { Connection, PublicKey } from "@solana/web3.js";
import { logger } from "./utils/logger.js";

// Mock the Client type since we are bypassing the import on Windows
type Client = any;
const CommitmentLevel = { PROCESSED: 0 };

export class GeyserClient {
    private grpcClient?: Client;
    private wssConnection?: Connection;
    private isWss: boolean = false;
    private subscriptionId?: number;

    constructor(endpoint: string, xToken?: string) {
        if (endpoint.startsWith("ws://") || endpoint.startsWith("wss://")) {
            this.isWss = true;
            // The HTTP endpoint is usually the WSS without wss://, replacing with https://
            const httpEndpoint = endpoint.replace("wss://", "https://").replace("ws://", "http://");
            this.wssConnection = new Connection(httpEndpoint, {
                wsEndpoint: endpoint
            });
            logger.info("GeyserClient initialized in FREE WebSocket (WSS) mode.");
        } else {
            logger.warn("gRPC mode is temporarily disabled on Windows due to NAPI native bindings. Use WSS for local testing or deploy to Ubuntu VPS.");
            // this.grpcClient = new Client(endpoint, xToken || "", undefined);
            // logger.info("GeyserClient initialized in PRO gRPC (Yellowstone) mode.");
        }
    }

    public async subscribe(programIdStr: string, callback: (tx: any) => void) {
        if (this.isWss && this.wssConnection) {
            const programId = new PublicKey(programIdStr);
            this.subscriptionId = this.wssConnection.onLogs(
                programId,
                (logs, ctx) => {
                    if (logs.err) return; // Ignore failed txs
                    // Mock a tx object since WSS onLogs only gives signatures and log arrays
                    const mockTx = {
                        signature: logs.signature,
                        logs: logs.logs,
                        isWssMock: true
                    };
                    callback(mockTx);
                },
                "processed"
            );
            logger.info(`WebSocket Subscription active for ${programIdStr}`);
        } else if (this.grpcClient) {
            // gRPC Logic bypassed for local testing
            logger.error("gRPC subscription called but client is not initialized.");
        }
    }

    public close() {
        logger.info("Fermeture de la connexion réseau...");
        try {
            if (this.isWss && this.wssConnection && this.subscriptionId !== undefined) {
                this.wssConnection.removeOnLogsListener(this.subscriptionId);
            } else if (this.grpcClient) {
                (this.grpcClient as any)?.close?.();
            }
        } catch (e) {
            logger.error(`Erreur lors de la fermeture: ${e}`);
        }
    }

    public getConnection(): Connection | undefined {
        return this.wssConnection;
    }
}
