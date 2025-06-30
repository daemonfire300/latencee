This is a bare-bones rust project and should contain a terminal TUI app which measures latency/speed to a list of given servers, e.g., google.com
with colorful indicators.
The idea is that this TUI can be run while on a plane or train WiFi to measure/record and visualize how stable it is right now.

Intermediary steps should be committed via git so that we can rollback to a given state and see how the project evolves.

Ideally the project uses smol instead of tokio or uses threads instead to avoid async overhead where-ever possible. If tokio is the best choice it's ok.

The project tries to use as little dependencies to other libraries as possible.

The project aims at running on MacOS and Linux.

The project should have a Dockerfile or Nix Flake to allow for CI/tests to run reproducibly.
