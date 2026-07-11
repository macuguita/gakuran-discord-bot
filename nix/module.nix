self:
{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.services.gakuran-bot;
  defaultUser = "gakuran-bot";
  inherit (lib)
    getExe
    literalExpression
    mkEnableOption
    mkIf
    mkOption
    mkPackageOption
    types
    ;
in
{
  options.services.gakuran-bot = {
    enable = mkEnableOption "gakuran-bot";
    package = mkPackageOption self.packages.${pkgs.stdenv.hostPlatform.system} "gakuran-bot" { };
    user = mkOption {
      description = ''
        User under which the service should run. If this is the default value,
        the user will be created, with the specified group as the primary
        group.
      '';
      type = types.str;
      default = defaultUser;
      example = literalExpression ''
        "bob"
      '';
    };
    group = mkOption {
      description = ''
        Group under which the service should run. If this is the default value,
        the group will be created.
      '';
      type = types.str;
      default = defaultUser;
      example = literalExpression ''
        "discordbots"
      '';
    };
    environmentFile = mkOption {
      description = ''
        Environment file as defined in {manpage}`systemd.exec(5)`.
        Use this to provide DISCORD_TOKEN (and any other secrets) without
        putting them in the world-readable Nix store.
      '';
      type = types.nullOr types.path;
      default = null;
      example = literalExpression ''
        "/run/agenix.d/1/gakuran-bot"
      '';
    };
  };
  config = mkIf cfg.enable {
    systemd.services."gakuran-bot" = {
      enable = true;
      wantedBy = [ "multi-user.target" ];
      after = [ "network.target" ];
      script = ''
        ${getExe cfg.package}
      '';
      serviceConfig = {
        Type = "simple";
        Restart = "on-failure";
        EnvironmentFile = mkIf (cfg.environmentFile != null) cfg.environmentFile;
        User = cfg.user;
        Group = cfg.group;

        # Persistent storage for reaction_roles.json / mod_log.json
        StateDirectory = "gakuran-bot";
        WorkingDirectory = "/var/lib/gakuran-bot";

        # hardening
        NoNewPrivileges = true;
        PrivateDevices = true;
        PrivateTmp = true;
        PrivateUsers = true;
        ProtectClock = true;
        ProtectControlGroups = true;
        ProtectHome = true;
        ProtectHostname = true;
        ProtectKernelLogs = true;
        ProtectKernelModules = true;
        ProtectKernelTunables = true;
        ProtectSystem = "strict";
        RestrictNamespaces = true;
        RestrictSUIDSGID = true;
        SystemCallArchitectures = "native";
        SystemCallFilter = [
          "@system-service"
          "~@resources"
          "~@privileged"
        ];
      };
    };
    users = {
      users = mkIf (cfg.user == defaultUser) {
        ${defaultUser} = {
          isSystemUser = true;
          inherit (cfg) group;
        };
      };
      groups = mkIf (cfg.group == defaultUser) { ${defaultUser} = { }; };
    };
  };
}
