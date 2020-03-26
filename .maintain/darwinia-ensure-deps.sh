#!/usr/bin/env bash


# This script is customized from Substrate to adapt to Darwinia project.

# The script is meant to check if the rules regarding packages
# dependencies are satisfied.
# The general format is:
# [top-lvl-dir] MESSAGE/[other-top-dir]

# For instance no crate within `./client` directory
# is allowed to import any crate with a directory path containing `frame`.
# Such rule is just: `client crates must not depend on anything in /frame`.

# The script should be run from the main repo directory!

set -u

# HARD FAILING
MUST_NOT=(
	"frame crates must not depend on anything in /bin/node"
	"primitives crates must not depend on anything in /frame"
)

VIOLATIONS=()
PACKAGES=()

function check_rule() {
	rule=$1
	from=$(echo $rule | cut -f1 -d\ )
	to=$(echo $rule | cut -f2- -d\/)

	# TODO: This is a good gitcoin issue
	case "$from" in 
		*/*)
			echo "darwinia-ensure-dep.sh is not ready to parsing $from and handle / "
			echo "please help us to do this like $to part"
			echo "read https://github.com/darwinia-network/darwinia/pull/401 to know more about it"
			exit 1
		;;
	esac

	cd $from
	echo "Checking rule '$rule'"
	packages=$(find -name Cargo.toml | xargs grep -wn "path.*\.\.\/$to")
	has_references=$(echo -n $packages | wc -c)
	if [ "$has_references" != "0" ]; then
		VIOLATIONS+=("$rule")
		# Find packages that violate:
		PACKAGES+=("$packages")
	fi
	cd - > /dev/null
}

for rule in "${MUST_NOT[@]}"
do
	check_rule "$rule";
done

I=0
for v in "${VIOLATIONS[@]}"
do
	cat << EOF

===========================================
======= Violation of rule: $v
===========================================
${PACKAGES[$I]}


EOF
	I=$I+1
done

exit $HARD_VIOLATIONS
