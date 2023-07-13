import Web3 from "web3";
import { describe } from "mocha";
import { step } from "mocha-steps";
import { expect } from "chai";
import { HOST_WS_URL, FAITH, FAITH_P, DEFAULT_GAS, customRequest } from "../config";
import { incrementerInfo } from "./contracts/contracts_info";
import { AbiItem } from "web3-utils";

const web3 = new Web3(HOST_WS_URL);
describe("Test nonce", () => {
	web3.eth.accounts.wallet.add(FAITH_P);
	const inc = new web3.eth.Contract(incrementerInfo.abi as AbiItem[]);
	inc.options.from = FAITH;
	inc.options.gas = DEFAULT_GAS;

	let init_nonce;
	step("Get the init nonce", async function () {
		init_nonce = await web3.eth.getTransactionCount(FAITH);
	});

	step("Increase nonce by 1 after transact create", async () => {
		let data = inc.deploy({ data: incrementerInfo.bytecode, arguments: [5] });
		let tx = await web3.eth.accounts.signTransaction(
			{
				from: FAITH,
				data: data.encodeABI(),
				gas: DEFAULT_GAS,
			},
			FAITH_P
		);
		let receipt = await web3.eth.sendSignedTransaction(tx.rawTransaction);

		expect(receipt.transactionHash).to.not.be.null;
		inc.options.address = receipt.contractAddress;

		expect(await web3.eth.getTransactionCount(FAITH)).to.be.equal(init_nonce + 1);
	}).timeout(60000);

	step("Increase nonce by 1 after contract call", async function () {
		let receipt = await inc.methods.reset().send();
		expect(receipt.transactionHash).to.not.be.null;

		expect(await web3.eth.getTransactionCount(FAITH)).to.be.equal(init_nonce + 2);
	}).timeout(60000);

	step("Rpc call doesn't update nonce", async function () {
		expect(await inc.methods.number().call()).to.be.equal("0");
		expect(await web3.eth.getTransactionCount(FAITH)).to.be.equal(init_nonce + 2);
	}).timeout(60000);

	step("Increase nonce by 1 after the transaction failed", async function () {
		await inc.methods
			.increment("0x1234")
			.send()
			.catch((err) =>
				expect(err.message).to.equal(
					`Returned error: VM Exception while processing transaction: revert.`
				)
			);
		expect(await web3.eth.getTransactionCount(FAITH)).to.be.equal(init_nonce + 3);
	}).timeout(60000);
});
