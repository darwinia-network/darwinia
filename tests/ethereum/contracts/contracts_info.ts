export const incrementerInfo = {
	bytecode:
		"608060405234801561001057600080fd5b506040516103963803806103968339818101604052810190610032919061007a565b80600081905550506100a7565b600080fd5b6000819050919050565b61005781610044565b811461006257600080fd5b50565b6000815190506100748161004e565b92915050565b6000602082840312156100905761008f61003f565b5b600061009e84828501610065565b91505092915050565b6102e0806100b66000396000f3fe608060405234801561001057600080fd5b50600436106100415760003560e01c80637cf5dab0146100465780638381f58a14610062578063d826f88f14610080575b600080fd5b610060600480360381019061005b9190610173565b61008a565b005b61006a610129565b60405161007791906101af565b60405180910390f35b61008861012f565b005b60038110156100ce576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004016100c590610227565b60405180910390fd5b806000546100dc9190610276565b600081905550803373ffffffffffffffffffffffffffffffffffffffff167fb182275171042022ff972a26edbd0171bccc74463bd22e56dbbeba4e93b7a66860405160405180910390a350565b60005481565b60008081905550565b600080fd5b6000819050919050565b6101508161013d565b811461015b57600080fd5b50565b60008135905061016d81610147565b92915050565b60006020828403121561018957610188610138565b5b60006101978482850161015e565b91505092915050565b6101a98161013d565b82525050565b60006020820190506101c460008301846101a0565b92915050565b600082825260208201905092915050565b7f7468652076616c7565206d7573742062652067726561746572207468616e2033600082015250565b60006102116020836101ca565b915061021c826101db565b602082019050919050565b6000602082019050818103600083015261024081610204565b9050919050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052601160045260246000fd5b60006102818261013d565b915061028c8361013d565b92508282019050808211156102a4576102a3610247565b5b9291505056fea2646970667358221220e92eb1e04daabd33e06f749d61351522829244ef4b25036e134bd214de2ed74864736f6c63430008130033",
	abi: [
		{
			inputs: [
				{
					internalType: "uint256",
					name: "_initialNumber",
					type: "uint256",
				},
			],
			stateMutability: "nonpayable",
			type: "constructor",
		},
		{
			anonymous: false,
			inputs: [
				{
					indexed: true,
					internalType: "address",
					name: "sender",
					type: "address",
				},
				{
					indexed: true,
					internalType: "uint256",
					name: "value",
					type: "uint256",
				},
			],
			name: "Increment",
			type: "event",
		},
		{
			inputs: [
				{
					internalType: "uint256",
					name: "_value",
					type: "uint256",
				},
			],
			name: "increment",
			outputs: [],
			stateMutability: "nonpayable",
			type: "function",
		},
		{
			inputs: [],
			name: "number",
			outputs: [
				{
					internalType: "uint256",
					name: "",
					type: "uint256",
				},
			],
			stateMutability: "view",
			type: "function",
		},
		{
			inputs: [],
			name: "reset",
			outputs: [],
			stateMutability: "nonpayable",
			type: "function",
		},
	],
};

export const opcodesInfo = {
	bytecode:
		"608060405234801561001057600080fd5b5060405161001d9061007e565b604051809103906000f080158015610039573d6000803e3d6000fd5b506000806101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff16021790555061008b565b6101438061052283390190565b6104888061009a6000396000f3fe608060405234801561001057600080fd5b506004361061004c5760003560e01c806355313dea146100515780636d3d14161461005b578063b9d1e5aa14610065578063f8a8fd6d1461006f575b600080fd5b610059610079565b005b61006361007b565b005b61006d610080565b005b610077610082565b005b005b600080fd5bfe5b600160021a6002f35b600581101561009f5760018101905061008b565b5060065b60058111156100b7576001810190506100a3565b5060015b60058112156100cf576001810190506100bb565b5060065b60058113156100e7576001810190506100d3565b506002156100f457600051505b60405160208101602060048337505060405160208101602060048339505060405160208101602060048360003c50503660005b8181101561013e5760028152600181019050610127565b505060008020506000602060403e6010608060106040610123612710fa506020610123600af05060008060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff169050600060405180807f697353616d654164647265737328616464726573732c61646472657373290000815250601e01905060405180910390209050600033905060405182815281600482015281602482015260648101604052602081604483600088611388f1505060405182815281600482015281602482015260648101604052602081604483600088611388f250506040518281528160048201528160248201526064810160405260208160448387611388f4505060006242004290507f50cb9fe53daa9737b786ab3646f04d0150dc50ef4e75f59509d83667ad5adb2060001b6040518082815260200191505060405180910390a07f50cb9fe53daa9737b786ab3646f04d0150dc50ef4e75f59509d83667ad5adb2060001b7f50cb9fe53daa9737b786ab3646f04d0150dc50ef4e75f59509d83667ad5adb2060001b6040518082815260200191505060405180910390a13373ffffffffffffffffffffffffffffffffffffffff1660001b7f50cb9fe53daa9737b786ab3646f04d0150dc50ef4e75f59509d83667ad5adb2060001b7f50cb9fe53daa9737b786ab3646f04d0150dc50ef4e75f59509d83667ad5adb2060001b6040518082815260200191505060405180910390a28060001b3373ffffffffffffffffffffffffffffffffffffffff1660001b7f50cb9fe53daa9737b786ab3646f04d0150dc50ef4e75f59509d83667ad5adb2060001b7f50cb9fe53daa9737b786ab3646f04d0150dc50ef4e75f59509d83667ad5adb2060001b6040518082815260200191505060405180910390a38060001b8160001b3373ffffffffffffffffffffffffffffffffffffffff1660001b7f50cb9fe53daa9737b786ab3646f04d0150dc50ef4e75f59509d83667ad5adb2060001b7f50cb9fe53daa9737b786ab3646f04d0150dc50ef4e75f59509d83667ad5adb2060001b6040518082815260200191505060405180910390a46002fffea265627a7a72315820da4feb2af5051e773c61e531dc7c451208bd898210e40f606667d91689c23c7c64736f6c63430005110032608060405234801561001057600080fd5b50610123806100206000396000f3fe6080604052348015600f57600080fd5b506004361060285760003560e01c8063161e715014602d575b600080fd5b608c60048036036040811015604157600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff169060200190929190803573ffffffffffffffffffffffffffffffffffffffff16906020019092919050505060a6565b604051808215151515815260200191505060405180910390f35b60008173ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff16141560e3576001905060e8565b600090505b9291505056fea265627a7a7231582082d761506d74e3b44f3c332693f36afc64d261352ea6bd6c457883eea792919064736f6c63430005110032",
	abi: [
		{
			inputs: [],
			payable: false,
			stateMutability: "nonpayable",
			type: "constructor",
		},
		{
			constant: false,
			inputs: [],
			name: "test",
			outputs: [],
			payable: false,
			stateMutability: "nonpayable",
			type: "function",
		},
		{
			constant: true,
			inputs: [],
			name: "test_invalid",
			outputs: [],
			payable: false,
			stateMutability: "view",
			type: "function",
		},
		{
			constant: true,
			inputs: [],
			name: "test_revert",
			outputs: [],
			payable: false,
			stateMutability: "view",
			type: "function",
		},
		{
			constant: true,
			inputs: [],
			name: "test_stop",
			outputs: [],
			payable: false,
			stateMutability: "view",
			type: "function",
		},
	],
};

export const eventInfo = {
	bytecode:
		"608060405234801561001057600080fd5b5061031b806100206000396000f3fe608060405234801561001057600080fd5b50600436106100885760003560e01c8063a67808571161005b578063a6780857146100b5578063b61c0503146100bf578063e8beef5b146100c9578063f38b0600146100d357610088565b8063102accc11461008d5780634e7ad3671461009757806365538c73146100a157806376bc21d9146100ab575b600080fd5b6100956100dd565b005b61009f610132565b005b6100a961014f565b005b6100b3610189565b005b6100bd6101bd565b005b6100c76101d6565b005b6100d1610214565b005b6100db61026c565b005b3373ffffffffffffffffffffffffffffffffffffffff16600115157f0e216b62efbb97e751a2ce09f607048751720397ecfb9eef1e48a6644948985b602a6040518082815260200191505060405180910390a3565b60011515602a6040518082815260200191505060405180910390a1565b7f65c9ac8011e286e89d02a269890f41d67ca2cc597b2c76c7c69321ff492be580602a6040518082815260200191505060405180910390a1565b3373ffffffffffffffffffffffffffffffffffffffff1660011515602a6040518082815260200191505060405180910390a2565b602a6040518082815260200191505060405180910390a0565b600115157f81933b308056e7e85668661dcd102b1f22795b4431f9cf4625794f381c271c6b602a6040518082815260200191505060405180910390a2565b7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff60001b3373ffffffffffffffffffffffffffffffffffffffff1660011515602a6040518082815260200191505060405180910390a3565b7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff60001b3373ffffffffffffffffffffffffffffffffffffffff16600115157f317b31292193c2a4f561cc40a95ea0d97a2733f14af6d6d59522473e1f3ae65f602a6040518082815260200191505060405180910390a456fea2646970667358221220153d53b462c5c7cf5d26f62987030b4c78e85c6c87a3b3e742769581438308c864736f6c634300060c0033",
	abi: [
		{
			inputs: [],
			stateMutability: "nonpayable",
			type: "constructor",
		},
		{
			anonymous: false,
			inputs: [
				{
					indexed: false,
					internalType: "uint256",
					name: "value",
					type: "uint256",
				},
			],
			name: "Log0",
			type: "event",
		},
		{
			anonymous: true,
			inputs: [
				{
					indexed: false,
					internalType: "uint256",
					name: "value",
					type: "uint256",
				},
			],
			name: "Log0Anonym",
			type: "event",
		},
		{
			anonymous: false,
			inputs: [
				{
					indexed: true,
					internalType: "bool",
					name: "aBool",
					type: "bool",
				},
				{
					indexed: false,
					internalType: "uint256",
					name: "value",
					type: "uint256",
				},
			],
			name: "Log1",
			type: "event",
		},
		{
			anonymous: true,
			inputs: [
				{
					indexed: true,
					internalType: "bool",
					name: "aBool",
					type: "bool",
				},
				{
					indexed: false,
					internalType: "uint256",
					name: "value",
					type: "uint256",
				},
			],
			name: "Log1Anonym",
			type: "event",
		},
		{
			anonymous: false,
			inputs: [
				{
					indexed: true,
					internalType: "bool",
					name: "aBool",
					type: "bool",
				},
				{
					indexed: true,
					internalType: "address",
					name: "aAddress",
					type: "address",
				},
				{
					indexed: false,
					internalType: "uint256",
					name: "value",
					type: "uint256",
				},
			],
			name: "Log2",
			type: "event",
		},
		{
			anonymous: true,
			inputs: [
				{
					indexed: true,
					internalType: "bool",
					name: "aBool",
					type: "bool",
				},
				{
					indexed: true,
					internalType: "address",
					name: "aAddress",
					type: "address",
				},
				{
					indexed: false,
					internalType: "uint256",
					name: "value",
					type: "uint256",
				},
			],
			name: "Log2Anonym",
			type: "event",
		},
		{
			anonymous: false,
			inputs: [
				{
					indexed: true,
					internalType: "bool",
					name: "aBool",
					type: "bool",
				},
				{
					indexed: true,
					internalType: "address",
					name: "aAddress",
					type: "address",
				},
				{
					indexed: true,
					internalType: "bytes32",
					name: "aBytes32",
					type: "bytes32",
				},
				{
					indexed: false,
					internalType: "uint256",
					name: "value",
					type: "uint256",
				},
			],
			name: "Log3",
			type: "event",
		},
		{
			anonymous: true,
			inputs: [
				{
					indexed: true,
					internalType: "bool",
					name: "aBool",
					type: "bool",
				},
				{
					indexed: true,
					internalType: "address",
					name: "aAddress",
					type: "address",
				},
				{
					indexed: true,
					internalType: "bytes32",
					name: "aBytes32",
					type: "bytes32",
				},
				{
					indexed: false,
					internalType: "uint256",
					name: "value",
					type: "uint256",
				},
			],
			name: "Log3Anonym",
			type: "event",
		},
		{
			inputs: [],
			name: "fireEventLog0",
			outputs: [],
			stateMutability: "nonpayable",
			type: "function",
		},
		{
			inputs: [],
			name: "fireEventLog0Anonym",
			outputs: [],
			stateMutability: "nonpayable",
			type: "function",
		},
		{
			inputs: [],
			name: "fireEventLog1",
			outputs: [],
			stateMutability: "nonpayable",
			type: "function",
		},
		{
			inputs: [],
			name: "fireEventLog1Anonym",
			outputs: [],
			stateMutability: "nonpayable",
			type: "function",
		},
		{
			inputs: [],
			name: "fireEventLog2",
			outputs: [],
			stateMutability: "nonpayable",
			type: "function",
		},
		{
			inputs: [],
			name: "fireEventLog2Anonym",
			outputs: [],
			stateMutability: "nonpayable",
			type: "function",
		},
		{
			inputs: [],
			name: "fireEventLog3",
			outputs: [],
			stateMutability: "nonpayable",
			type: "function",
		},
		{
			inputs: [],
			name: "fireEventLog3Anonym",
			outputs: [],
			stateMutability: "nonpayable",
			type: "function",
		},
	],
};

export const blsInfo = {
	abi: [
		{
			inputs: [
				{
					internalType: "bytes[]",
					name: "pubkeys",
					type: "bytes[]",
				},
				{
					internalType: "bytes",
					name: "message",
					type: "bytes",
				},
				{
					internalType: "bytes",
					name: "sig",
					type: "bytes",
				},
			],
			name: "fast_aggregate_verify",
			outputs: [
				{
					internalType: "bool",
					name: "",
					type: "bool",
				},
			],
			stateMutability: "nonpayable",
			type: "function",
		},
	],
};
