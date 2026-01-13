{
  inputs.flakelight-rust.url = "github:accelbread/flakelight-rust";
  outputs =
    { flakelight-rust, ... }:
    flakelight-rust ./. {

      devShell =
        pkgs:
        let
          packages = with pkgs; [
            bacon
            cargo-watch
            pkg-config
          ];
          libraries = with pkgs; [
            libxkbcommon
            wayland
            udev
            alsa-lib
            vulkan-loader
            dbus
          ];
        in
        {
          packages = packages ++ libraries;
          env.LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath libraries;
        };

    };
}
