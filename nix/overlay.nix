{
  config,
  inputs,
  ...
}: {
  flake.overlays.default = final: _: let
    # TODO convert tools from submodule to javascript dependency
    # Pull submodule from specific remote commit
    # TODO update package-lock.json
    # Seem to be version mismatches? also submodule package-lock.json doesn't seem to be resolved
    # TODO add deno types as a dependency rather than generating them on the fly
    # Nix sandbox prevents running network code (i.e. installing deno types from github URL)
    # NOTE ideally we would have src = inputs.self
    src = let
      inherit (final) runCommand lib fetchFromGitHub fetchurl nodejs gnused;
      tools = fetchFromGitHub {
        owner = "project-gauntlet";
        repo = "tools";
        rev = "7bc5ef7d8326172b4353d37763b3c55e4ace051f";
        hash = "sha256-1JvHcqbIG6+Dp/CHeX/tOBPKuUpLBnGMWzrfYBZWSD8=";
      };
      deno-types = fetchurl {
        url = "https://github.com/denoland/deno/releases/download/v1.36.4/lib.deno.d.ts";
        hash = "sha256-faimw0TezsJVH8yYUJYS5BZ6FNJ3Ly2doru3AFuC68k=";
      };
    in
      runCommand "source" {} ''
        export PATH="${lib.makeBinPath (with final; [gnused jq moreutils])}:$PATH"
        mkdir -p tmp/{,tools,js/deno/dist}

        # Merge in submodule
        cp -R ${inputs.self}/* tmp
        cp -R ${tools}/* tmp/tools

        # Provide updated and consolidated lockfile
        cp -f ${./package-lock.json} tmp/package-lock.json

        # Implement @project-gauntlet/deno stub generation
        sed 's:/// <reference lib="deno\..*" />::g' ${deno-types} > tmp/js/deno/dist/lib.deno.d.ts
        jq '.scripts.["run-generator-source"] |= ""' tmp/js/deno/package.json | sponge tmp/js/deno/package.json

        # patch gauntlet build tool shebang
        jq ".scripts.build += \" && ${lib.getExe gnused} --in-place '1s:.*:#!${lib.getExe nodejs}:' ./bin/main.js && cat ./bin/main.js\"" tmp/tools/package.json | sponge tmp/tools/package.json

        cp -R tmp $out
      '';
    inherit
      (final)
      # Libraries
      lib
      stdenv
      # Builders
      fetchurl
      importNpmLock
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
    inherit (lib) concatStringsSep getExe' makeBinPath makeLibraryPath optional optionalString;
    inherit (stdenv.hostPlatform) isLinux rust system;
    # Borrowed from other packages in nixpkgs https://github.com/search?q=repo%3ANixOS%2Fnixpkgs%20RUSTY_V8_ARCHIVE&type=code
    buildRustyV8Url = version: target: "https://github.com/denoland/rusty_v8/releases/download/v${version}/librusty_v8_release_${target}.a";
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
      echo "Update RUSTY_V8_ARCHIVE version and hashes as follows:"
      echo "version: $version"
      for system in ${concatStringsSep " " config.systems}; do
          target=$(nix eval --raw nixpkgs#legacyPackages.$system.stdenv.hostPlatform.rust.rustcTarget)
          hash=$(nix-prefetch-url --print-path ${buildRustyV8Url "$version" "$target"} | tail -n 1 | xargs nix hash file)
          echo "$system: $hash"
      done
    '';
    cargoArtifactsArgs = {
      inherit src;
      pname = "gauntlet";
      version = "v11";
      cargoExtraArgs = "--features release";
      nativeBuildInputs = [cmake pkg-config protobuf];
      buildInputs = [openssl];
      # OPENSSL_CONFIG_DIR didn't work for vendored dependencies
      OPENSSL_NO_VENDOR = true;
      RUSTY_V8_ARCHIVE = fetchRustyV8 "0.74.3" {
        x86_64-linux = "sha256-8pa8nqA6rbOSBVnp2Q8/IQqh/rfYQU57hMgwU9+iz4A=";
        aarch64-darwin = "sha256-Djnuc3l/jQKvBf1aej8LG5Ot2wPT0m5Zo1B24l1UHsM=";
        aarch64-linux = "sha256-3kXOV8rlCNbNBdXgOtd3S94qO+JIKyOByA4WGX+XVP0=";
        x86_64-darwin = "sha256-iBBVKZiSoo08YEQ8J/Rt1/5b7a+2xjtuS6QL/Wod5nQ=";
      };
    };
    craneLib = inputs.crane.mkLib final;
    cargoArtifacts = craneLib.buildDepsOnly cargoArtifactsArgs;
  in {
    gauntlet = craneLib.buildPackage {
      inherit (cargoArtifactsArgs) cargoExtraArgs pname src version OPENSSL_NO_VENDOR RUSTY_V8_ARCHIVE;
      inherit cargoArtifacts;
      npmDeps = importNpmLock {npmRoot = src;};
      nativeBuildInputs = cargoArtifactsArgs.nativeBuildInputs ++ [nodejs importNpmLock.npmConfigHook] ++ optional isLinux makeWrapper;
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
      postFixup = optionalString isLinux ''
        patchelf --add-rpath ${makeLibraryPath [libxkbcommon libGL xorg.libX11 wayland]} $out/bin/gauntlet
        wrapProgram $out/bin/gauntlet --suffix PATH : ${makeBinPath [gtk3]}
        substituteInPlace $out/lib/systemd/user/gauntlet.service --replace /usr/bin/gauntlet $out/bin/gauntlet
      '';
      passthru = {inherit fetchRustyV8Hashes;};
      meta.mainProgram = "gauntlet";
    };
  };
}
