import Web3 from "web3";
import { describe } from "mocha";
import { step } from "mocha-steps";
import { expect } from "chai";
import {
	HOST_HTTP_URL,
	FAITH,
	FAITH_P,
	BLOCK_GAS_LIMIT,
	customRequest,
	DEFAULT_GAS,
} from "../config";
import { incrementerInfo } from "./contracts/contracts_info";
import { AbiItem } from "web3-utils";

const web3 = new Web3(HOST_HTTP_URL);
describe("Test transaction gas limit", () => {
	web3.eth.accounts.wallet.add(FAITH_P);
	const inc = new web3.eth.Contract(incrementerInfo.abi as AbiItem[]);
	const data = inc.deploy({ data: incrementerInfo.bytecode, arguments: [5] });

	it("Test contract create estimate gas", async () => {
		expect(
			await web3.eth.estimateGas({
				from: FAITH,
				data: data.encodeABI(),
			})
		).to.equal(257336);
	}).timeout(60000);

	it("Test contract call estimate gas", async () => {
		let tx = await web3.eth.accounts.signTransaction(
			{
				from: FAITH,
				data: data.encodeABI(),
				gas: DEFAULT_GAS,
			},
			FAITH_P
		);
		let receipt = await web3.eth.sendSignedTransaction(tx.rawTransaction);
		inc.options.address = receipt.contractAddress;

		expect(await inc.methods.increment(3).estimateGas()).to.equal(28506);
	}).timeout(60000);

	it("Test transaction gas limit < `BLOCK_GAS_LIMIT`", async () => {
		let tx = await web3.eth.accounts.signTransaction(
			{
				from: FAITH,
				data: data.encodeABI(),
				gas: BLOCK_GAS_LIMIT - 1,
			},
			FAITH_P
		);
		const receipt = await customRequest(web3, "eth_sendRawTransaction", [tx.rawTransaction]);

		expect((receipt as any).transactionHash).to.be.not.null;
	}).timeout(60000);

	it("Test transaction gas limit = `BLOCK_GAS_LIMIT`", async () => {
		let tx = await web3.eth.accounts.signTransaction(
			{
				from: FAITH,
				data: data.encodeABI(),
				gas: BLOCK_GAS_LIMIT,
			},
			FAITH_P
		);
		const receipt = await customRequest(web3, "eth_sendRawTransaction", [tx.rawTransaction]);

		expect((receipt as any).transactionHash).to.be.not.null;
	}).timeout(60000);

	it("Test transaction gas limit > `BLOCK_GAS_LIMIT`", async () => {
		let tx = await web3.eth.accounts.signTransaction(
			{
				from: FAITH,
				data: data.encodeABI(),
				gas: BLOCK_GAS_LIMIT + 1,
			},
			FAITH_P
		);
		const receipt = await customRequest(web3, "eth_sendRawTransaction", [tx.rawTransaction]);

		expect((receipt as any).error.message).to.equal("exceeds block gas limit");
	}).timeout(60000);
});
