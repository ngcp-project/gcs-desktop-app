{
  description = "Tauri development environment";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };

  outputs =
    { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        nativeBuildInputs = with pkgs; [
          pkg-config
          cargo-tauri
          rustc
          cargo
          bun
          docker-compose
        ];

        buildInputs = with pkgs; [
          webkitgtk_4_1
          gtk3
          cairo
          gdk-pixbuf
          glib
          dbus
          openssl_3
          librsvg
        ];

        # Required for Tauri to find libraries at runtime
        LD_LIBRARY_PATH = "${pkgs.webkitgtk_6_0}/lib:${pkgs.gtk3}/lib";

        # Optional: Workaround for some rendering issues
        WEBKIT_DISABLE_COMPOSITING_MODE = "1";

        # Needed on Wayland to render correctly
        shellHook = ''
          export XDG_DATA_DIRS="$XDG_DATA_DIRS:$GSETTINGS_SCHEMAS_PATH"
        '';

      };
    };
}
