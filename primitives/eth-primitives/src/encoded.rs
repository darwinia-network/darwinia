use sp_std::vec::Vec;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header(Vec<u8>);
impl Header {
	/// Create a new owning header view.
	/// Expects the data to be an RLP-encoded header -- any other case will likely lead to
	/// panics further down the line.
	pub fn new(encoded: Vec<u8>) -> Self {
		Header(encoded)
	}
}
