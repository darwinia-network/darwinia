import Web3 from "web3";
import { describe } from "mocha";
import { expect } from "chai";
import { HOST_WS_URL, CHAIN_ID, ALITH } from "../config";

const web3 = new Web3(HOST_WS_URL);
describe("Test constants RPC", () => {
	it("Should have 0 hashrate", async () => {
		expect(await web3.eth.getHashrate()).to.equal(0);
	});

	it("Should have chainId", async () => {
		expect(await web3.eth.getChainId()).to.equal(CHAIN_ID);
	});

	it("block author should be ALITH", async () => {
		expect(await web3.eth.getCoinbase()).to.equal(ALITH);
	});
});
