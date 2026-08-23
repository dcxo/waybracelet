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
      {

        package = lib.mkForce (
          {
            pkgs,
            defaultMeta,
            ...
          }:
          pkgs.rustPlatform.buildRustPackage {
            pname = "waybracelet";
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
                "iced_exdevtools-0.19.1" = "sha256-39ha/5Kjot8DEJEZtsCocQoa7+gFt/EOymZMJmWeV5M=";
                "mothscheme-0.1.0" = "sha256-UWIyKWAU4Ierlxj5jOa9OnwLVPBCbCY7IsNCFaCj+Js=";
              };
            };
            strictDeps = true;
            meta = defaultMeta;
            env.NIX_LDFLAGS = "-rpath ${pkgs.lib.makeLibraryPath (libraries pkgs)}";
            dontPatchELF = true;
          }
        );

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
