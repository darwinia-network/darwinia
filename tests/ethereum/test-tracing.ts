import Web3 from "web3";
import { describe } from "mocha";
import { step } from "mocha-steps";
import { expect } from "chai";
import { HOST_WS_URL, FAITH, FAITH_P, DEFAULT_GAS, customRequest } from "../config";
import { incrementerInfo } from "./contracts/contracts_info";
import { AbiItem } from "web3-utils";

const web3 = new Web3(HOST_WS_URL);
describe("Test EVM tracing", () => {
	web3.eth.accounts.wallet.add(FAITH_P);
	const inc = new web3.eth.Contract(incrementerInfo.abi as AbiItem[]);
	inc.options.from = FAITH;
	inc.options.gas = DEFAULT_GAS;

	let transact_hash;
	let transact_receipt;
	let filter_begin;
	step("Deploy contract first", async () => {
		let data = inc.deploy({ data: incrementerInfo.bytecode, arguments: [5] });
		let tx = await web3.eth.accounts.signTransaction(
			{
				from: FAITH,
				data: data.encodeABI(),
				gas: DEFAULT_GAS,
			},
			FAITH_P
		);
		transact_receipt = await web3.eth.sendSignedTransaction(tx.rawTransaction);
		filter_begin = transact_receipt.blockNumber;

		expect(transact_receipt.transactionHash).to.not.be.null;
		inc.options.address = transact_receipt.contractAddress;
	}).timeout(60000);

	step("Increase number", async function () {
		transact_receipt = await inc.methods.increment(3).send();
		transact_hash = transact_receipt.transactionHash;

		expect(transact_receipt.transactionHash).to.not.be.null;
		expect(await inc.methods.number().call()).to.be.equal("8");
	}).timeout(60000);

	step("RPC debug_traceTransaction should work", async function () {
		let trace_result = await customRequest(web3, "debug_traceTransaction", [transact_hash]);
		expect(trace_result.result.structLogs.length).to.be.equal(198);
		expect(trace_result.result.structLogs[0].depth).to.be.equal(1);
		expect(trace_result.result.structLogs[0].pc).to.be.equal(0);
	}).timeout(60000);

	step("RPC debug_traceBlockByNumber should work", async function () {
		let block_number = web3.utils.toHex(transact_receipt.blockNumber);
		let trace_result = await customRequest(web3, "debug_traceBlockByNumber", [
			block_number,
			{ tracer: "callTracer" },
		]);

		expect(trace_result.result[0].result.from).to.be.equal(FAITH.toLowerCase());
		expect(trace_result.result[0].result.to).to.be.equal(inc.options.address.toLowerCase());
		expect(trace_result.result[0].result.type).to.be.equal("CALL");
	}).timeout(60000);

	step("RPC debug_traceBlockByHash should work", async function () {
		let block_hash = web3.utils.toHex(transact_receipt.blockHash);
		let trace_result = await customRequest(web3, "debug_traceBlockByHash", [
			block_hash,
			{ tracer: "callTracer" },
		]);

		expect(trace_result.result[0].result.from).to.be.equal(FAITH.toLowerCase());
		expect(trace_result.result[0].result.to).to.be.equal(inc.options.address.toLowerCase());
		expect(trace_result.result[0].result.type).to.be.equal("CALL");
	}).timeout(60000);

	step("RPC trace_filter should work", async function () {
		let trace_result = await customRequest(web3, "trace_filter", [
			{
				fromBlock: web3.utils.toHex(filter_begin),
				toBlock: web3.utils.toHex(transact_receipt.blockNumber),
				fromAddress: [FAITH],
			},
		]);

		expect(trace_result.result.length).to.equal(2);
	}).timeout(60000);
});
