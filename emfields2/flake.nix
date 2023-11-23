{
  description = "LaTeX Document Demo";
  inputs = {
    nixpkgs.url = github:NixOS/nixpkgs/nixos-unstable;
    flake-utils.url = github:numtide/flake-utils;
  };
  outputs = { self, nixpkgs, flake-utils }:
    with flake-utils.lib; eachSystem allSystems (system:
    let
      pkgs = nixpkgs.legacyPackages.${system};
      tex = pkgs.texlive.combine {
        inherit (pkgs.texlive) scheme-minimal latex-bin latexmk xetex
        algorithmicx algorithms amsmath babel-english babel-greek greek-fontenc hyphen-greek gfsartemisia gfsneohellenic noto fontspec bookmark caption cite
        colortbl enumitem epstopdf epstopdf-pkg float footmisc grfext hyperref
        import infwarerr kvdefinekeys kvoptions kvsetkeys listings ltxcmds
        mathtools nicematrix oberdiek pgf thmtools tools varwidth lm-math;
      };
    in rec {
      packages = {
        document = pkgs.stdenvNoCC.mkDerivation rec {
          name = "latex-demo-document";
          src = self;
          buildInputs = [ pkgs.coreutils tex ];
          phases = ["unpackPhase" "buildPhase" "installPhase"];
          buildPhase = ''
            export PATH="${pkgs.lib.makeBinPath buildInputs}";
            mkdir -p .cache/texmf-var
            cd ex1;
            env TEXMFHOME=.cache TEXMFVAR=.cache/texmf-var \
              latexmk -interaction=nonstopmode -pdf -xelatex \
              main.tex
          '';
          installPhase = ''
            mkdir -p $out
            cp main.pdf $out/
          '';
        };
      };
      defaultPackage = packages.document;
    });
}
