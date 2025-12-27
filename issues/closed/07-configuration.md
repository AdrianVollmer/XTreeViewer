# Configuration File

Add support for a configuration file to allow users to customize XTV
behavior and appearance. Create a configuration file format (TOML or
YAML) that supports settings like color theme (dark/light), streaming
file size threshold, default expanded depth, and keyboard shortcuts. The
config file should be loaded from `~/.config/xtv/config.toml` (XDG
standard). Implement a default configuration with sensible values that
is used when no config file exists. Add validation for config values and
helpful error messages for invalid configurations. Consider adding a
`--config` CLI flag to specify a custom config file path.
