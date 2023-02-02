import Web3 from "web3";
import { describe } from "mocha";
import { step } from "mocha-steps";
import { expect } from "chai";
import { HOST_HTTP_URL, FAITH, FAITH_P, EXTRINSIC_GAS_LIMIT, customRequest } from "../config";
import { incrementerInfo } from "./contracts/contracts_info";
import { AbiItem } from "web3-utils";

const web3 = new Web3(HOST_HTTP_URL);
describe("Test transaction gas limit", () => {
	web3.eth.accounts.wallet.add(FAITH_P);
	const inc = new web3.eth.Contract(incrementerInfo.abi as AbiItem[]);
	const data = inc.deploy({ data: incrementerInfo.bytecode, arguments: [5] });

	it("Test transaction gas limit < `EXTRINSIC_GAS_LIMIT`", async () => {
		let tx = await web3.eth.accounts.signTransaction(
			{
				from: FAITH,
				data: data.encodeABI(),
				gas: EXTRINSIC_GAS_LIMIT - 1,
			},
			FAITH_P
		);
		const receipt = await customRequest(web3, "eth_sendRawTransaction", [tx.rawTransaction]);

		expect((receipt as any).transactionHash).to.be.not.null;
	}).timeout(60000);

	it("Test transaction gas limit = `EXTRINSIC_GAS_LIMIT`", async () => {
		let tx = await web3.eth.accounts.signTransaction(
			{
				from: FAITH,
				data: data.encodeABI(),
				gas: EXTRINSIC_GAS_LIMIT,
			},
			FAITH_P
		);
		const receipt = await customRequest(web3, "eth_sendRawTransaction", [tx.rawTransaction]);

		expect((receipt as any).transactionHash).to.be.not.null;
	}).timeout(60000);

	it("Test transaction gas limit > `EXTRINSIC_GAS_LIMIT`", async () => {
		let tx = await web3.eth.accounts.signTransaction(
			{
				from: FAITH,
				data: data.encodeABI(),
				gas: EXTRINSIC_GAS_LIMIT + 1,
			},
			FAITH_P
		);
		const receipt = await customRequest(web3, "eth_sendRawTransaction", [tx.rawTransaction]);

		expect((receipt as any).error.message).to.equal("exceeds block gas limit");
	}).timeout(60000);
});
