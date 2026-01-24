{
  inputs.flakelight-rust.url = "github:accelbread/flakelight-rust";
  outputs =
    { flakelight-rust, ... }:
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
            naersk,
            pkgs,
            defaultMeta,
            ...
          }:
          naersk.buildPackage {
            nativeBuildInputs = libraries pkgs;
            buildInputs = libraries pkgs;
            src = lib.fileset.toSource {
              root = src;
              inherit (config) fileset;
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
