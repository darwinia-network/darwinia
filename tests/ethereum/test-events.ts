import Web3 from "web3";
import { describe } from "mocha";
import { step } from "mocha-steps";
import { expect } from "chai";
import { HOST_WS_URL, FAITH, FAITH_P, DEFAULT_GAS } from "../config";
import { eventInfo } from "./contracts/contracts_info";
import { AbiItem } from "web3-utils";

const web3 = new Web3(HOST_WS_URL);
describe("Test event", () => {
	web3.eth.accounts.wallet.add(FAITH_P);
	const event = new web3.eth.Contract(eventInfo.abi as AbiItem[]);
	event.options.from = FAITH;
	event.options.gas = DEFAULT_GAS;

	step("Deploy event contract", async function () {
		let tx = await web3.eth.accounts.signTransaction(
			{
				from: FAITH,
				data: eventInfo.bytecode,
				gas: DEFAULT_GAS,
			},
			FAITH_P
		);
		let receipt = await web3.eth.sendSignedTransaction(tx.rawTransaction);

		expect(receipt.transactionHash).to.not.be.null;
		event.options.address = receipt.contractAddress;
	}).timeout(60000);

	step("Fire event log0", async function () {
		await event.methods.fireEventLog0().send();

		event.once("Log0", function (error, event) {
			expect(event.raw.data).to.be.equal(
				"0x000000000000000000000000000000000000000000000000000000000000002a"
			);
			expect(event.signature).to.be.equal(
				"0x65c9ac8011e286e89d02a269890f41d67ca2cc597b2c76c7c69321ff492be580"
			);
		});
	}).timeout(60000);

	step("Fire event log0 anonym", async function () {
		await event.methods.fireEventLog0Anonym().send();

		event.once("Log0", function (error, event) {
			expect(event.raw.data).to.be.equal(
				"0x000000000000000000000000000000000000000000000000000000000000002a"
			);
			expect(event.signature).to.be.null;
		});
	}).timeout(60000);

	step("Fire event log3", async function () {
		await event.methods.fireEventLog3().send();

		event.once("Log0", function (error, event) {
			expect(event.raw.data).to.be.equal(
				"0x000000000000000000000000000000000000000000000000000000000000002a"
			);
			expect(event.raw.topics.length).to.be.equal(4);
			expect(event.raw.topics[1]).to.be.equal(
				"0x0000000000000000000000000000000000000000000000000000000000000001"
			);
			expect(event.raw.topics[2]).to.be.equal(
				"0x0000000000000000000000006be02d1d3665660d22ff9624b7be0551ee1ac91b"
			);
			expect(event.raw.topics[3]).to.be.equal(
				"0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
			);
			expect(event.signature).to.be.equal(
				"0x317b31292193c2a4f561cc40a95ea0d97a2733f14af6d6d59522473e1f3ae65f"
			);
		});
	}).timeout(60000);

	step("Fire event log3 anonym", async function () {
		await event.methods.fireEventLog3Anonym().send();

		event.once("Log0", function (error, event) {
			expect(event.raw.data).to.be.equal(
				"0x000000000000000000000000000000000000000000000000000000000000002a"
			);
			expect(event.raw.topics.length).to.be.equal(3);
			expect(event.raw.topics[0]).to.be.equal(
				"0x0000000000000000000000000000000000000000000000000000000000000001"
			);
			expect(event.raw.topics[1]).to.be.equal(
				"0x0000000000000000000000006be02d1d3665660d22ff9624b7be0551ee1ac91b"
			);
			expect(event.raw.topics[2]).to.be.equal(
				"0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
			);
			expect(event.signature).to.be.null;
		});
	}).timeout(60000);
});
