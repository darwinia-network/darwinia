import Web3 from "web3";
import { describe } from "mocha";
import { expect } from "chai";
import { HOST_HTTP_URL, CHAIN_ID } from "../config";

const web3 = new Web3(HOST_HTTP_URL);
describe("Test constants RPC", () => {
	it("Should have 0 hashrate", async () => {
		expect(await web3.eth.getHashrate()).to.equal(0);
	});

	it("Should have chainId", async () => {
		expect(await web3.eth.getChainId()).to.equal(CHAIN_ID);
	});

	// TODO: FIX ME
	it.skip("block author should be 0x0000000000000000000000000000000000000000", async () => {
		// This address `0x1234567890` is hardcoded into the runtime find_author
		// as we are running manual sealing consensus.
		expect(await web3.eth.getCoinbase()).to.equal("0x0000000000000000000000000000000000000000");
	});
});
