// darwinia
use crate::*;

impl<S> Processor<S> {
	pub fn process_parachain_system(&mut self) -> &mut Self {
		if self.test {
			return self;
		}

		// Storage items.
		// https://github.com/darwinia-network/darwinia-2.0/issues/275#issuecomment-1427725708
		// https://github.com/paritytech/cumulus/blob/09418fc04c2608b123f36ca80f16df3d2096753b/pallets/parachain-system/src/lib.rs#L582-L595
		let last_dmq_mqc_head_key =
			"0x45323df7cc47150b3930e2666b0aa313911a5dd3f1155f5b7d0c5aa102a757f9";
		let last_hrmp_mqc_heads_key =
			"0x45323df7cc47150b3930e2666b0aa3133dca42deb008c6559ee789c9b9f70a2c";
		let mut last_dmq_mqc_head = String::new();
		let mut last_hrmp_mqc_heads = String::new();

		log::info!(
			"take para `ParachainSystem::LastDmqMqcHead` and `ParachainSystem::LastHrmpMqcHeads`"
		);

		self.para_state
			.take_raw_value(last_dmq_mqc_head_key, &mut last_dmq_mqc_head)
			.take_raw_value(last_hrmp_mqc_heads_key, &mut last_hrmp_mqc_heads);

		log::info!("set `ParachainSystem::LastDmqMqcHead` and `ParachainSystem::LastHrmpMqcHeads`");
		self.shell_state
			.insert_raw_key_raw_value(last_dmq_mqc_head_key.into(), last_dmq_mqc_head)
			.insert_raw_key_raw_value(last_hrmp_mqc_heads_key.into(), last_hrmp_mqc_heads);

		self
	}
}
