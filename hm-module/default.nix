{ self, ... }:
{ config, pkgs, lib, ... }:
let
  cfg = config.programs.waystart;
  waystartPkg = self.packages.${pkgs.stdenv.hostPlatform.system}.default;
  tomlFormat = pkgs.formats.toml { };
in
{
  options.programs.waystart = {
    enable = lib.mkEnableOption "Waystart, a start menu for Wayland-based window managers";
    withDaemon = lib.mkOption {
      default = true;
      example = true;
      description = "Whether to enable the daemon service.";
      type = lib.types.bool;
    };

    package = lib.mkOption {
      type = with lib.types; nullOr package;
      default = waystartPkg;
      description = "Waystart package to use.";
    };

    settings = lib.mkOption {
      inherit (tomlFormat) type;
      default = { };
      description = "Configuration settings for Waystart.";
    };

    systemdTarget = lib.mkOption {
      type = lib.types.str;
      default = config.wayland.systemd.target;
      defaultText = lib.literalExpression "config.wayland.systemd.target";
      example = "hyprland-session.target";
      description = "Systemd target to bind to.";
    };
  };

  config = lib.mkIf cfg.enable {
    home.packages = [ cfg.package ];

    xdg.configFile."waystart.toml" = lib.mkIf (cfg.settings != { }) {
      source = tomlFormat.generate "waystart-config" cfg.settings;
    };

    systemd.user.services.waystart = lib.mkIf (cfg.package != null && cfg.withDaemon) {
      Install = {
        WantedBy = [ cfg.systemdTarget ];
      };

      Unit = {
        ConditionEnvironment = "WAYLAND_DISPLAY";
        Description = "Waystart daemon";
        After = [ cfg.systemdTarget ];
        PartOf = [ cfg.systemdTarget ];
        X-Restart-Triggers = lib.mkIf (cfg.settings != { }) [
          "${config.xdg.configFile."waystart.toml".source}"
        ];
      };

      Service = {
        ExecStart = "${lib.getExe cfg.package} daemon";
        Restart = "always";
        RestartSec = "10";
      };
    };
  };
}
