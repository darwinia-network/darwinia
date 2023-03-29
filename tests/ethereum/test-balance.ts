import Web3 from "web3";
import { describe } from "mocha";
import { step } from "mocha-steps";
import { expect } from "chai";
import { HOST_HTTP_URL, FAITH, FAITH_P, ETHAN, DEFAULT_GAS } from "../config";

const web3 = new Web3(HOST_HTTP_URL);
describe("Test balances", () => {
	const VALUE = "0x200";
	const GAS_PRICE = "0x12309ce54000"; // 20_000_000_000_000

	let init_from;
	let init_to;
	step("Account has correct balance", async function () {
		init_from = await web3.eth.getBalance(FAITH);
		init_to = await web3.eth.getBalance(ETHAN);
		expect(Number(init_from)).to.be.greaterThan(Number(VALUE));
	});

	step("Balance should be updated after transfer", async function () {
		let tx = await web3.eth.accounts.signTransaction(
			{
				from: FAITH,
				to: ETHAN,
				value: VALUE,
				gasPrice: GAS_PRICE,
				gas: "0x100000",
			},
			FAITH_P
		);
		await web3.eth.sendSignedTransaction(tx.rawTransaction);
	}).timeout(60000);

	step("Balance should be updated after transfer", async function () {
		const expectedFromBalance =
			BigInt(init_from) - BigInt(21000) * BigInt(GAS_PRICE) - BigInt(VALUE);
		expect(await web3.eth.getBalance(FAITH)).to.equal(expectedFromBalance.toString());

		const expectedToBalance = (BigInt(init_to) + BigInt(VALUE)).toString();
		expect(await web3.eth.getBalance(ETHAN)).to.equal(expectedToBalance);
	});
});
