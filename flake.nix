{
	inputs = {
		nixpkgs.url = "nixpkgs/nixos-unstable";
		flake-utils.url = "github:numtide/flake-utils";
	};

	outputs = { self, nixpkgs, flake-utils, ... }:
	flake-utils.lib.eachDefaultSystem (system:
		let 
			pkgs = import nixpkgs { inherit system; };
		in {
			packages.default = pkgs.rustPlatform.buildRustPackage {
				pname = "fb2epub";
				version = "0.2.1";

				src = ./.;

				cargoLock.lockFile = ./Cargo.lock;
			};

			devShells.default = pkgs.mkShell {
				buildInputs = with pkgs; [
					rustc
					cargo

					clippy
					rust-analyzer
				];

				shellHook = ''
					export HOME="$PWD/.nix-cache"
					export CARGO_TERM_COLOR=always
				'';
			};
		}
	);
}
