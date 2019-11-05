use blake2::Digest;
// for macro vec![]
use rstd::vec;
use rstd::vec::Vec;

const ALL_ONES: usize = usize::max_value();

pub type Hash = Vec<u8>;

pub fn peak_map_height(mut index: usize) -> (usize, usize) {
	if index == 0 {
		return (0, 0);
	}

	let mut peak_size = ALL_ONES >> index.leading_zeros();
	let mut bitmap = 0;
	while peak_size != 0 {
		bitmap <<= 1;
		if index >= peak_size {
			index -= peak_size;
			bitmap |= 1;
		}

		peak_size >>= 1;
	}

	(bitmap, index)
}

pub fn peak_indexes(size: usize) -> Vec<usize> {
	if size == 0 {
		return vec![];
	}

	let mut peak_size = ALL_ONES >> size.leading_zeros();
	let mut num_left = size;
	let mut sum_prev_peaks = 0;
	let mut peaks = vec![];

	while peak_size != 0 {
		if num_left >= peak_size {
			sum_prev_peaks += peak_size;
			num_left -= peak_size;

			peaks.push(sum_prev_peaks - 1);
		}

		peak_size >>= 1;
	}

	if num_left > 0 {
		vec![]
	} else {
		peaks
	}
}

#[inline]
pub fn is_leaf(index: usize) -> bool {
	bintree_height(index) == 0
}

#[inline]
pub fn bintree_height(index: usize) -> usize {
	if index == 0 {
		0
	} else {
		peak_map_height(index).1
	}
}

pub fn family_branch(index: usize, last_index: usize) -> Vec<(usize, usize)> {
	let (peak_map, height) = peak_map_height(index);
	let mut peak = 1 << height;
	let mut branch = vec![];
	let mut current = index;
	let mut sibling;
	while current < last_index {
		if (peak_map & peak) != 0 {
			current += 1;
			sibling = current - 2 * peak;
		} else {
			current += 2 * peak;
			sibling = current - 1;
		}
		if current > last_index {
			break;
		}

		branch.push((current, sibling));
		peak <<= 1;
	}

	branch
}

pub fn family(index: usize) -> (usize, usize) {
	let (peak_map, height) = peak_map_height(index);
	let peak = 1 << height;

	if (peak_map & peak) != 0 {
		(index + 1, index + 1 - 2 * peak)
	} else {
		(index + 2 * peak, index + 2 * peak - 1)
	}
}

#[inline]
pub fn is_left_sibling(index: usize) -> bool {
	let (peak_map, height) = peak_map_height(index);
	let peak = 1 << height;
	(peak_map & peak) == 0
}

#[inline]
pub fn leaf_index(n: usize) -> usize {
	if n == 0 {
		0
	} else {
		2 * n - n.count_ones() as usize
	}
}

#[inline]
pub fn chain_two_hash<D, H>(left: H, right: H) -> Hash
where
	D: Digest,
	H: AsRef<[u8]>,
{
	D::new().chain(left).chain(right).result().to_vec()
}
