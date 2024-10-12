import Web3 from "web3";
import { describe } from "mocha";
import { step } from "mocha-steps";
import { expect } from "chai";
import { HOST_WS_URL, FAITH, FAITH_P, DEFAULT_GAS } from "../config";
import { incrementerInfo } from "./contracts/contracts_info";
import { AbiItem } from "web3-utils";

const web3 = new Web3(HOST_WS_URL);
describe("Test contract", () => {
	web3.eth.accounts.wallet.add(FAITH_P);
	const inc = new web3.eth.Contract(incrementerInfo.abi as AbiItem[]);
	inc.options.from = FAITH;
	inc.options.gas = DEFAULT_GAS;

	let transact_hash;
	step("Deploy contract", async () => {
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
	}).timeout(60000);

	step("Get contract code", async function () {
		expect(await web3.eth.getCode(inc.options.address)).not.to.be.null;
	});

	step("Get default number", async function () {
		expect(await inc.methods.number().call()).to.be.equal("5");
	});

	step("Get default number in pending block", async function () {
		const result = await web3.eth.call(
			{
				to: inc.options.address,
				data: inc.methods.number().encodeABI(),
			},
			"pending"
		);
		expect(web3.utils.hexToNumberString(result)).to.be.equal("5");
	});

	step("Increase number", async function () {
		let receipt = await inc.methods.increment(3).send();
		transact_hash = receipt.transactionHash;

		expect(receipt.transactionHash).to.not.be.null;
		expect(await inc.methods.number().call()).to.be.equal("8");
	}).timeout(60000);

	step("Transaction bloom and Block bloom", async function () {
		// transaction bloom
		let receipt = await web3.eth.getTransactionReceipt(transact_hash);
		expect(web3.utils.isInBloom(receipt.logsBloom, receipt.logs[0].address)).to.be.true;
		for (let topic of receipt.logs[0].topics) {
			expect(web3.utils.isInBloom(receipt.logsBloom, topic)).to.be.true;
		}

		// block bloom
		let block = await web3.eth.getBlock(receipt.blockHash);
		expect(web3.utils.isInBloom(block.logsBloom, receipt.logs[0].address)).to.be.true;
		for (let topic of receipt.logs[0].topics) {
			expect(web3.utils.isInBloom(block.logsBloom, topic)).to.be.true;
		}
	});

	step("Reset number", async function () {
		let receipt = await inc.methods.reset().send();

		expect(receipt.transactionHash).to.not.be.null;
		expect(await inc.methods.number().call()).to.be.equal("0");
	}).timeout(60000);

	step("Get correct revert reason", async function () {
		await inc.methods
			.increment(2)
			.call()
			.catch((err) => {
				expect(err.message).to.equal(
					`Returned error: VM Exception while processing transaction: revert the value must be greater than 3`
				);
			});
	}).timeout(60000);
});
