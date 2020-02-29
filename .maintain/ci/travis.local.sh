readonly TEST_CRATES=(
    'kton'
    'ring'
    'staking'
    'treasury'
);

function main() {
    for crate in ${TEST_CRATES[@]}
    do
	cargo test -p "darwinia-$crate"
    done
}

main
