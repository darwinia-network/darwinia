let
  mozillaOverlay =
    import (builtins.fetchGit {
      url = "https://github.com/mozilla/nixpkgs-mozilla.git";
      rev = "78e723925daf5c9e8d0a1837ec27059e61649cb6";
    });
  nixpkgs = import <nixpkgs> { overlays = [ mozillaOverlay ]; };
  rust-nightly = with nixpkgs; (nixpkgs.latest.rustChannels.nightly.rust);
in
with nixpkgs; pkgs.mkShell {
  nativeBuildInputs = [
    rust-nightly
  ];
  buildInputs = [
    clang
    rocksdb
    pkg-config
    openssl.dev
  ] ++ lib.optionals stdenv.isDarwin [
    darwin.apple_sdk.frameworks.Security
  ];

  RUST_SRC_PATH = "${rust-nightly}/lib/rustlib/src/rust/src";
  LIBCLANG_PATH = "${llvmPackages.libclang.lib}/lib";
  PROTOC = "${protobuf}/bin/protoc";
  ROCKSDB_LIB_DIR = "${rocksdb}/lib";
}
