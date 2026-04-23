{ self }:
{ config, lib, pkgs, ... }:

with lib;

let
  cfg = config.programs.wardex;
  yamlFormat = pkgs.formats.yaml { };
in
{
  options.programs.wardex = {
    enable = mkEnableOption "wardex";

    package = mkOption {
      type = types.package;
      default = self.packages.${pkgs.system}.default;
      defaultText = literalExpression "flake.packages.\${pkgs.system}.default";
      description = "The wardex package to install.";
    };

    settings = mkOption {
      type = yamlFormat.type;
      default = { };
      example = literalExpression ''
        {
          paths = {
            workspace = "/home/user/workspace";
          };
        }
      '';
      description = ''
        Configuration written to {file}`$XDG_CONFIG_HOME/wardex/config.yaml`.
      '';
    };

    enableBashIntegration = mkOption {
      type = types.bool;
      default = true;
      description = "Whether to enable Bash integration (completions).";
    };

    enableZshIntegration = mkOption {
      type = types.bool;
      default = true;
      description = "Whether to enable Zsh integration (completions).";
    };
  };

  config = mkIf cfg.enable {
    home.packages = [ cfg.package ];

    xdg.configFile."wardex/config.yaml" = mkIf (cfg.settings != { }) {
      source = yamlFormat.generate "wardex-config.yaml" cfg.settings;
    };

    # Dynamic completions via CompleteEnv — includes both subcommand/flag
    # completion and runtime value completion (event names, categories).
    # Supersedes static completions from `wardex completions <shell>`.
    programs.bash.initExtra = mkIf cfg.enableBashIntegration ''
      source <(COMPLETE=bash ${cfg.package}/bin/wardex)
    '';

    programs.zsh.initContent = mkIf cfg.enableZshIntegration (mkAfter ''
      source <(COMPLETE=zsh ${cfg.package}/bin/wardex)
    '');
  };
}
