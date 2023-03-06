// --- paritytech ---
#[allow(unused)]
use frame_support::{migration, traits::OnRuntimeUpgrade, weights::Weight};
// --- darwinia-network ---
#[allow(unused)]
use crate::*;

pub struct CustomOnRuntimeUpgrade;
impl OnRuntimeUpgrade for CustomOnRuntimeUpgrade {
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<(), &'static str> {
		Ok(())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade() -> Result<(), &'static str> {
		Ok(())
	}

	fn on_runtime_upgrade() -> Weight {
		migrate()
	}
}

fn migrate() -> Weight {
	panic!(
		r#"
                     ,,,                     ,,,
                   ,,,,,,  ,,,           ,,   ,,,,,
                  ,,,,,,,,,,,,,         ,,,,,(,,,,,,
        ,,,        ,,,,,,,,,,,.         ,,,,,,,,,,,,        ,,,
      ,,,,,,,       ,,,,,,,,,            .,,,,,,,,,       ,,,,,,,
       ,,,,,,                  ,,,,,,,,,                  ,,,,,,
.,,,,,    ,,,,            ,,,,,,,,/ ,,,,,,,,,            ,,,*    ,,,,,
,,,,,,      ,,,,       ,,,,,              .,,,,.       ,,,,      ,,,,,.
   .,,,       ,,,,,,,,,,,,   .,,,,,,,,,,,    ,,,,,,,,,,,,      .,,,
     ,,,,            ,,,   ,,,,,,,,,,,,,,,,    ,,,            ,,,,
       ,,,,,,,,,,,,,,,,.  ,,,,,,,,,,,,,,,,,,   ,,,,,,,,,,,,,,,,,
                    ,,,   ,,,,,,,,,,,,,,,,,,,   ,,,
                    ,,,.  ,,,,,,,,,,,,,,,,,,   ,,,.
       ,,,,,,,,,,,,,,,,,.   ,,,,,,,,,,,,,,,   ,,,,,,,,,,,,,,,,,,
     ,,,,             ,,,,    ,,,,,,,,,,.    ,,,,             ,,,,
   ,,,,.      ,,,,,,,,,,,,,,,             ,,,,,,,,,,,,,,,      *,,,.
,,,,,,      ,,,,           ,,,,,,,,,,,,,,,,,           ,,,,      ,,,,,,
,,,,,,    ,,,,                  .,,,,.*                  ,,,.    ,,,,,
       ,,,,,,                                             ,,,,,,
      ,,,,,,,                                             ,,,,,,,
        ,,,                                                 ,,,

             .d8888b.                  888      d888
             d88P  Y88b                 888     d8888
             888    888                 888       888
             888        888d888 8888b.  88888b.   888
             888        888P"      "88b 888 "88b  888
             888    888 888    .d888888 888  888  888
             Y88b  d88P 888    888  888 888 d88P  888
              "Y8888P"  888    "Y888888 88888P" 8888888
    .d8888b.  888                                            888
   d88P  Y88b 888                                            888
   Y88b.      888                                            888
    "Y888b.   888888 .d88b.  88888b.  88888b.   .d88b.   .d88888
       "Y88b. 888   d88""88b 888 "88b 888 "88b d8P  Y8b d88" 888
         "888 888   888  888 888  888 888  888 88888888 888  888
   Y88b  d88P Y88b. Y88..88P 888 d88P 888 d88P Y8b.     Y88b 888
    "Y8888P"   "Y888 "Y88P"  88888P"  88888P"   "Y8888   "Y88888
                             888      888
                             888      888
                             888      888

         Crab1 and Crab Parachain1 are merged into Crab2.
        Check: https://github.com/darwinia-network/darwinia
"#
	);

	// 0
	// RuntimeBlockWeights::get().max_block
}
