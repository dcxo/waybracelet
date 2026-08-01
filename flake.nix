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
                "iced_exdevtools-0.19.1" = "sha256-d6ZqbOId+lr8kIL8t41CzdoVhVxmzA4vHU2Z+HkChSc=";
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
