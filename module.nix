{
  config,
  lib,
  ...
}: let
  cfg = config.services.nowplaying;
in {
  options.services.nowplaying = {
    enable = lib.mkEnableOption "nowplaying last.fm websocket server";

    package = lib.mkOption {
      type = lib.types.package;
      description = "the nowplaying package to use";
    };

    host = lib.mkOption {
      type = lib.types.str;
      default = "127.0.0.1";
      description = "address to bind to. defaults to localhost; put a reverse proxy in front for public access.";
    };

    port = lib.mkOption {
      type = lib.types.port;
      default = 3000;
      description = "port to listen on";
    };

    lastfmUser = lib.mkOption {
      type = lib.types.str;
      example = "tetrafox_";
      description = "last.fm username whose currently-playing track is broadcast";
    };

    environmentFile = lib.mkOption {
      type = lib.types.path;
      description = ''
        path to an env file containing `LASTFM_API_KEY=...`. systemd reads it
        as root before dropping privileges, so the file should be root-owned
        and mode 0400. use sops/agenix to render this so the secret never
        lands in the nix store.
      '';
    };

    openFirewall = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = "open `port` in the firewall";
    };
  };

  config = lib.mkIf cfg.enable {
    systemd.services.nowplaying = {
      description = "nowplaying last.fm websocket server";
      after = ["network-online.target"];
      wants = ["network-online.target"];
      wantedBy = ["multi-user.target"];

      environment = {
        LASTFM_USER = cfg.lastfmUser;
        HOST = cfg.host;
        PORT = toString cfg.port;
      };

      serviceConfig = {
        ExecStart = lib.getExe cfg.package;
        EnvironmentFile = cfg.environmentFile;
        Restart = "on-failure";
        RestartSec = 10;

        DynamicUser = true;

        # hardening - small rust webserver, doesn't need much.
        AmbientCapabilities = "";
        CapabilityBoundingSet = "";
        LockPersonality = true;
        MemoryDenyWriteExecute = true;
        NoNewPrivileges = true;
        PrivateDevices = true;
        PrivateTmp = true;
        ProtectClock = true;
        ProtectControlGroups = true;
        ProtectHome = true;
        ProtectHostname = true;
        ProtectKernelLogs = true;
        ProtectKernelModules = true;
        ProtectKernelTunables = true;
        ProtectProc = "invisible";
        ProtectSystem = "strict";
        RestrictAddressFamilies = ["AF_INET" "AF_INET6"];
        RestrictNamespaces = true;
        RestrictRealtime = true;
        RestrictSUIDSGID = true;
        SystemCallArchitectures = "native";
        SystemCallFilter = ["@system-service" "~@privileged" "~@resources"];
      };
    };

    networking.firewall = lib.mkIf cfg.openFirewall {
      allowedTCPPorts = [cfg.port];
    };
  };
}
