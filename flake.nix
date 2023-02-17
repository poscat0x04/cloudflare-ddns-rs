{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-22.11";
    flake-utils.url = "github:poscat0x04/flake-utils";
    nix-filter.url = "github:numtide/nix-filter";
  };
  outputs =
    { self
    , nixpkgs
    , flake-utils
    , nix-filter
    }:
  flake-utils.eachDefaultSystem (system:
    let
      pkgs = import nixpkgs { inherit system; overlays = [ self.overlay ]; };
    in {
      packages = rec {
        inherit (pkgs) cloudflare-ddns;
        default = cloudflare-ddns;
      };
    }) // {
      nixosModules.cloudflare-ddns =
        { config, lib, pkgs, ... }:
        let
          cfg = config.services.cloudflare-ddns;
          configFile = pkgs.writeText "cloudflare-ddns-config.toml" ''
            name = "${cfg.config.name}"
            if_name = "${cfg.config.interface}"
            zone_id = "${cfg.config.zoneId}"
            v4 = ${lib.boolToString cfg.config.ipv4}
            v6 = ${lib.boolToString cfg.config.ipv6}
            proxied = ${lib.boolToString cfg.config.proxied}
            ttl = ${builtins.toString cfg.config.ttl}
          '';
          devUnit = "sys-subsystem-net-devices-${cfg.config.interface}.device";
          devDep = lib.optional cfg.bindToInterface devUnit;
        in {
          options.services.cloudflare-ddns = with lib; {
            enable = mkEnableOption ''
              Cloudflare dynamic DNS service.
            '';

            bindToInterface = mkOption {
              type = types.bool;
              default = false;
              description = ''
                Whether to bind to the interface. When bound to an interface,
                the service will only be started on the creation of that interface
                and brought down on the deletion of that interface. This mode is
                suitable if the IP address of said interface does not change after
                it has been assigned, as is the case of, for example, PPPoE.

                The alternative is to start the service on a timer, which, can be
                configured using the calendar option.
              '';
            };

            calendar = mkOption {
              type = types.str;
              default = "hourly";
            };

            config = {
              name = mkOption {
                type = types.str;
              };

              interface = mkOption {
                type = types.str;
              };

              zoneId = mkOption {
                type = types.str;
              };

              ipv4 = mkOption {
                type = types.bool;
                default = true;
              };

              ipv6 = mkOption {
                type = types.bool;
                default = true;
              };

              proxied = mkOption {
                type = types.bool;
                default = false;
              };

              ttl = mkOption {
                type = types.int;
                default = 120;
              };
            };
          };

          config = lib.mkIf cfg.enable {
            nixpkgs.overlays = lib.mkAfter [ self.overlay ];

            systemd.services.cloudflare-ddns = {
              wantedBy = devDep;
              after = [ "network-online.target" "nss-lookup.target" ] ++ devDep;
              bindsTo = devDep;
              requisite = lib.optional (!cfg.bindToInterface) devUnit;
              unitConfig = {
                # prevent excessive restarting when run by a timer
                StartLimitIntervalSec = lib.mkIf (!cfg.bindToInterface) "1h";
              };
              serviceConfig = {
                DynamicUser = true;
                User = "cloudflare-ddns";
                Group = "cloudflare-ddns";
                SystemCallArchitectures = [ "native" ];
                ProtectClock = true;
                ProtectControlGroups = true;
                ProtectHome = true;
                ProtectHostname = true;
                ProtectKernelLogs = true;
                ProtectKernelModules = true;
                ProtectKernelTunables = true;
                ProtectProc = "noaccess";
                RestrictAddressFamilies = [ "AF_UNIX" "AF_INET" "AF_INET6" "AF_NETLINK" ];
                RestrictNamespaces = true;
                Restart = "on-failure";
                # don't restart if the user messed up their config
                RestartPreventExitStatus = [ "2" ];
                RestartSec = "3s";
                Type = "oneshot";
                RemainAfterExit = cfg.bindToInterface;
                # wait for 5 seconds before actually calling cf-ddns, should be enough time for the interface
                # to configure its IP address
                ExecStartPre = "/run/current-system/sw/bin/sleep 5";
                ExecStart = "${pkgs.cloudflare-ddns}/bin/cf-ddns --config ${configFile}";
              };
            };

            systemd.timers.cloudflare-ddns = lib.mkIf (!cfg.bindToInterface) {
              wantedBy = [ "timers.target" ];
              timerConfig = {
                OnCalendar = cfg.calendar;
                Unit = "cloudflare-ddns.service";
              };
            };
          };
        };
      overlay = final: prev: {
        cloudflare-ddns = with final.rustPlatform; buildRustPackage {
          pname = "cloudflare-ddns";
          version = "0.2.0";

          src = nix-filter.lib {
            root = ./.;
            include = [
              ./src
              ./Cargo.toml
              ./Cargo.lock
            ];
          };
          cargoLock.lockFile = ./Cargo.lock;

          nativeBuildInputs = [ final.pkg-config ];
          buildInputs = [ final.openssl final.systemd.dev ];
        };
      };
    };
}
