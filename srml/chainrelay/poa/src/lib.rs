pub struct BestHeader<Hash> {
	height: u64, // enough for ethereum poa network (kovan)
	hash: Hash,
}
