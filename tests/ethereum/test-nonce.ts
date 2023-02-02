import Web3 from "web3";
import { describe } from "mocha";
import { step } from "mocha-steps";
import { expect } from "chai";
import { HOST_HTTP_URL, FAITH, FAITH_P } from "../config";

const web3 = new Web3(HOST_HTTP_URL);
describe("Test balances", () => {
	let init_nonce;
	step("Get the init nonce", async function () {
		init_nonce = await web3.eth.getTransactionCount(FAITH);
	});

	step("Make a transaction", async function () {
		let tx = await web3.eth.accounts.signTransaction(
			{
				from: FAITH,
				to: "0x1111111111111111111111111111111111111111",
				value: 0x200,
				gasPrice: "0x3B9ACA00", // 1000000000,
				gas: "0x100000",
			},
			FAITH_P
		);
		await web3.eth.sendSignedTransaction(tx.rawTransaction);
	}).timeout(60000);

	step("Nonce should be updated after transaction", async function () {
		expect(await web3.eth.getTransactionCount(FAITH)).to.be.equal(init_nonce + 1);
	});
});
