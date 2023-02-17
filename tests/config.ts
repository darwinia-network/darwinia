import Web3 from "web3";
import { JsonRpcResponse } from "web3-core-helpers";

export const CHAIN_ID = 43;
export const HOST_HTTP_URL = "http://127.0.0.1:9933";
export const HOST_WS_URL = "ws://127.0.0.1:9944";
// TODO: https://github.com/darwinia-network/darwinia-2.0/issues/248
export const BLOCK_GAS_LIMIT = 9375000;
export const DEFAULT_GAS = 4000000;
// EXTRINSIC_GAS_LIMIT = [BLOCK_GAS_LIMIT - BLOCK_GAS_LIMIT * (NORMAL_DISPATCH_RATIO - AVERAGE_ON_INITIALIZE_RATIO) - EXTRINSIC_BASE_Weight] / WEIGHT_PER_GAS  = (1_000_000_000_000 * 2 * (0.75-0.025) - 125_000_000) / 40_000
export const EXTRINSIC_GAS_LIMIT = 9059375;

// Accounts builtin
export const ALITH = "0xf24ff3a9cf04c71dbc94d0b566f7a27b94566cac";
export const FAITH = "0xC0F0f4ab324C46e55D02D0033343B4Be8A55532d";
export const FAITH_P = "0xb9d2ea9a615f3165812e8d44de0d24da9bbd164b65c4f0573e1ce2c8dbd9c8df";

export async function customRequest(web3: Web3, method: string, params: any[]) {
	return new Promise<JsonRpcResponse>((resolve, reject) => {
		(web3.currentProvider as any).send(
			{
				jsonrpc: "2.0",
				id: 1,
				method,
				params,
			},
			(error: Error | null, result?: JsonRpcResponse) => {
				if (error) {
					reject(
						`Failed to send custom request (${method} (${params.join(",")})): ${
							error.message || error.toString()
						}`
					);
				}
				resolve(result);
			}
		);
	});
}
