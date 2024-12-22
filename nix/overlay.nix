{
  config,
  inputs,
  ...
}: {
  flake.overlays.default = final: _: let
    # These must be updated following the instructions in ./nix/README.md when dependencies are updated or the version bumped
    version = "v12";
    npmDepsHash = "sha256-TlfUwNsmyN4dzqBh3CW33pGXxBZHLhSDyAqS4fJCmPU=";
    RUSTY_V8_ARCHIVE = fetchRustyV8 "130.0.2" {
      aarch64-darwin = "sha256-aWZ/4Q4Wttx37xOdBmTCPGP+eYGhr4CM1UkYq8pC7Qs=";
      aarch64-linux = "sha256-p9+tHmKIM5wBABubHIAstpwfzO19ypPzOuaV4b6loCU=";
      x86_64-darwin = "sha256-zNC0DAkMbbFM1M+t6rgKtN0QAm4ONEbCi6Sxivhf8dk=";
      x86_64-linux = "sha256-ew2WZhdsHfffRQtif076AWAlFohwPo/RbmW/6D3LzkU=";
    };

    inherit
      (final)
      # Libraries
      lib
      stdenv
      # Builders
      buildPackages
      fetchurl
      fetchNpmDeps
      makeWrapper
      writeShellScriptBin
      # Packages
      cmake
      deno
      gtk3
      libxkbcommon
      libGL
      xorg
      nodejs
      openssl
      pkg-config
      protobuf
      wayland
      yq
      ;
    inherit (buildPackages.npmHooks.override {inherit nodejs;}) npmConfigHook;
    inherit (lib) concatStringsSep getExe' makeBinPath makeLibraryPath optional;
    inherit (stdenv.hostPlatform) isLinux rust system;
    # Borrowed from other packages in nixpkgs https://github.com/search?q=repo%3ANixOS%2Fnixpkgs%20RUSTY_V8_ARCHIVE&type=code
    buildRustyV8Url = version: target: "https://github.com/denoland/rusty_v8/releases/download/v${version}/librusty_v8_release_${target}.a.gz";
    fetchRustyV8 = version: hashes:
      fetchurl {
        name = "librusty_v8-${version}";
        url = buildRustyV8Url version rust.rustcTarget;
        hash = hashes.${system};
        meta.version = version;
        meta.sourceProvenance = [lib.sourceTypes.binaryNativeCode];
      };
    fetchRustyV8Hashes = writeShellScriptBin "fetch-librusty_v8-hashes.sh" ''
      version=$(${getExe' yq "tomlq"} '.package | map(select(.name == "v8")) | .[0].version' --raw-output ${inputs.self}/Cargo.lock)
      echo "Update ./nix/overlay.nix as follows:"
      echo "RUSTY_V8_ARCHIVE = fetchRustyV8 \"$version\" {"
      for system in ${concatStringsSep " " config.systems}; do
          target=$(nix eval --raw nixpkgs#legacyPackages.$system.stdenv.hostPlatform.rust.rustcTarget)
          hash=$(nix-prefetch-url --print-path ${buildRustyV8Url "$version" "$target"} | tail -n 1 | xargs nix hash file)
          echo "  $system = \"$hash\";"
      done
      echo "};"
    '';
    pname = "gauntlet";
    src = inputs.self;
    cargoArtifactsArgs = {
      inherit pname src version RUSTY_V8_ARCHIVE;
      cargoExtraArgs = "--features release";
      nativeBuildInputs = [cmake pkg-config protobuf];
      buildInputs = [openssl];
      # OPENSSL_CONFIG_DIR didn't work for vendored dependencies
      OPENSSL_NO_VENDOR = true;
    };
    craneLib = inputs.crane.mkLib final;
    cargoArtifacts = craneLib.buildDepsOnly cargoArtifactsArgs;
  in {
    gauntlet = craneLib.buildPackage {
      inherit (cargoArtifactsArgs) cargoExtraArgs OPENSSL_NO_VENDOR RUSTY_V8_ARCHIVE;
      inherit cargoArtifacts pname src version;
      # fetchNpmDeps + makeCacheWritable is necessary with npm git:// dependencies
      npmDeps = fetchNpmDeps {
        inherit src;
        name = "${pname}-${version}-npm-deps";
        hash = npmDepsHash;
      };
      makeCacheWritable = true;
      nativeBuildInputs = cargoArtifactsArgs.nativeBuildInputs ++ [nodejs npmConfigHook] ++ optional isLinux makeWrapper;
      buildInputs = cargoArtifactsArgs.buildInputs ++ [deno];
      preBuild = "npm run build";
      postInstall =
        if isLinux
        then ''
          install -Dm644 assets/linux/gauntlet.desktop $out/share/applications/gauntlet.desktop
          install -Dm644 assets/linux/gauntlet.service $out/lib/systemd/user/gauntlet.service
          install -Dm644 assets/linux/icon_256.png $out/share/icons/hicolor/256x256/apps/gauntlet.png
        ''
        else ''
          contentsDir=$out/Applications/Gauntlet.app/Contents
          install -Dm755 $out/bin/gauntlet $contentsDir/MacOS/Gauntlet
          install -Dm644 assets/macos/AppIcon.icns $contentsDir/Resources/AppIcon.icns
          install -Dm644 assets/macos/Info.plist $contentsDir/Info.plist
        '';
      postFixup =
        if isLinux
        then ''
          patchelf --add-rpath ${makeLibraryPath [libxkbcommon libGL xorg.libX11 wayland]} $out/bin/gauntlet
          wrapProgram $out/bin/gauntlet --suffix PATH : ${makeBinPath [gtk3]}
          substituteInPlace $out/lib/systemd/user/gauntlet.service --replace /usr/bin/gauntlet $out/bin/gauntlet
        ''
        else ''
          substituteInPlace $out/Applications/Gauntlet.app/Contents/Info.plist --replace __VERSION__ ${version}
        '';
      passthru = {inherit fetchRustyV8Hashes;};
      meta.mainProgram = "gauntlet";
    };
  };
}
