{
  inputs.flakelight-rust.url = "github:accelbread/flakelight-rust";
  outputs =
    { flakelight-rust, self, ... }:
    let
      libraries =
        pkgs: with pkgs; [
          libxkbcommon
          wayland
          udev
          alsa-lib
          vulkan-loader
          dbus
        ];
    in
    flakelight-rust ./. (
      {
        lib,
        src,
        config,
        ...
      }:
      let
        mkCrate =
          {
            pkgs,
            defaultMeta,
            pname,
            subdir ? null,
          }:
          pkgs.rustPlatform.buildRustPackage (
            {
              inherit pname;
              version = "0.1.0";
              nativeBuildInputs = with pkgs; [ pkg-config ];
              buildInputs = libraries pkgs;
              src = lib.fileset.toSource {
                root = src;
                fileset = ./.;
              };
              cargoLock = {
                lockFile = ./Cargo.lock;
                outputHashes = {
                  "mothscheme-0.1.0" = "sha256-UWIyKWAU4Ierlxj5jOa9OnwLVPBCbCY7IsNCFaCj+Js=";
                };
              };
              strictDeps = true;
              meta = defaultMeta;
              env.NIX_LDFLAGS = "-rpath ${pkgs.lib.makeLibraryPath (libraries pkgs)}";
              dontPatchELF = true;
            }
            // (if subdir == null then { } else { buildAndTestSubdir = subdir; })
          );
      in
      {

        package = lib.mkForce (
          {
            pkgs,
            defaultMeta,
            ...
          }:
          mkCrate {
            inherit pkgs defaultMeta;
            pname = "waybracelet";
          }
        );

        packages.spotlight =
          {
            pkgs,
            defaultMeta,
            ...
          }:
          mkCrate {
            inherit pkgs defaultMeta;
            pname = "spotlight";
            subdir = "shells/spotlight";
          };

        devShell =
          pkgs:
          let
            packages = with pkgs; [
              bacon
              cargo-watch
              pkg-config
            ];
            librariesS = libraries pkgs;
          in
          {
            packages = packages ++ librariesS;
            env.LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath librariesS;
          };

      }
    );
}
