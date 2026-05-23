# which-key.wayland

[[中文](docs/README_zh.md)]

---

A key-hint panel for Wayland, inspired by the Neovim plugin [which-key.nvim](https://github.com/folke/which-key.nvim)
and the [Helix](https://helix-editor.com/) editor style.

`which-key.wayland` displays a key-hint overlay on Wayland compositors that support the `wlr_layer_shell` protocol,
allowing you to navigate and execute commands via key sequences.

## Installation

### Cargo

```sh
cargo install which-key-wayland
```

### Nix Flake

#### :snowflake: Quick Try

If you have Nix installed or are using NixOS, you can run it directly:

```sh
nix run github:liuhq/which-key.wayland
```

#### NixOS Installation

Add it to your `flake.nix`:

```nix
{
  inputs = {
    which-key-wayland.url = "github:liuhq/which-key.wayland";
  };

  outputs = { self, nixpkgs, which-key-wayland, ... }@inputs: {
    nixosConfigurations.<your-host> = nixpkgs.lib.nixosSystem {
      modules = [
        ({ ... }: {
          environment.systemPackages = [
            which-key-wayland.packages.x86_64-linux.which-key-wayland
          ];
        })
      ];
    };
  };
}
```

### Build from Source

Build requirements:

- Rust toolchain (`rustc`, `cargo`)
- `libxkbcommon`, `wayland` (runtime and build dependencies)
- `pkg-config` (build tool)

```sh
# clone
git clone https://github.com/liuhq/which-key.wayland.git
cd which-key.wayland
# build
cargo build --release
# run
./target/release/which-key-wayland
```

## Usage

The program runs as a background daemon, communicating over the D-Bus session bus.

```sh
which-key-wayland       # start the program and show the panel (wakes up if already running)
which-key-wayland show  # send show command to the running instance
which-key-wayland quit  # quit the program
```

It is recommended to bind a hotkey to `which-key-wayland` in your window manager/compositor. The program automatically
handles first launch and subsequent invocations.

Examples:

**Niri** (`$XDG_CONFIG_HOME/niri/config.kdl`):

```kdl
binds {
    Mod+Space { spawn "which-key-wayland"; }
}
```

**Hyprland** (`$XDG_CONFIG_HOME/hypr/hyprland.conf`):

```text
bind = $mainMod, Space, exec, which-key-wayland
```

## Configuration

Configuration file uses the [KDL](https://kdl.dev/) format.

### Priority

Configuration files are read in the following order of priority:

1. Path specified by the `$WKW_CONFIG_FILE` environment variable
2. `$XDG_CONFIG_HOME/which-key-wayland/config.kdl`
3. `$HOME/.config/which-key-wayland/config.kdl` (fallback when `XDG_CONFIG_HOME` is not set)
4. If none of the above exist or parsing fails, defaults are used

### Configuration Example

See [`examples/config.kdl`](./examples/config.kdl).

### Configuration Options

#### `timeout`

Time (in milliseconds) before the panel auto-hides when idle. Set to `0` to never auto-hide.

| Field     | Type    | Default | Description          |
|-----------|---------|---------|----------------------|
| `timeout` | integer | `2000`  | ms, `0` to disable   |

#### `font` Block

| Field         | Type  | Default | Description  |
|---------------|-------|---------|--------------|
| `size`        | float | `16.0`  | font size    |
| `line-height` | float | `20.0`  | line height  |

#### `color` Block

Color values support three hex formats: `"#RGB"`, `"#RRGGBB"`, `"#RRGGBBAA"` (`#`
optional). The last two digits represent transparency (alpha); omitted means fully opaque.

| Field          | Type   | Default     | Description                |
|----------------|--------|-------------|----------------------------|
| `fg-key`       | string | `"#D8DEE9"` | key text color             |
| `fg-separator` | string | `"#4C566A"` | separator color            |
| `fg-action`    | string | `"#88C0D0"` | `Action` description color |
| `fg-group`     | string | `"#5E81AC"` | `Group` description color  |
| `bg`           | string | `"#2E3440"` | background color           |

#### `layout` Block

| Field       | Type    | Default | Description                                                                   |
|-------------|---------|---------|-------------------------------------------------------------------------------|
| `width`     | integer | `500`   | panel width (pixels)                                                          |
| `max-items` | integer | `10`    | maximum items per page                                                        |
| `padding`   | integer | `4`     | padding (pixels)                                                              |
| `radius`    | integer | `0`     | corner radius (pixels)                                                        |
| `anchor`    | integer | `2`     | screen anchor: `1` top-right, `2` bottom-right, `3` bottom-left, `4` top-left |

##### `margin` Sub-block

| Field    | Type    | Default | Description    |
|----------|---------|---------|----------------|
| `top`    | integer | `0`     | top margin     |
| `right`  | integer | `0`     | right margin   |
| `bottom` | integer | `0`     | bottom margin  |
| `left`   | integer | `0`     | left margin    |

#### `bind` Block

Key binding configuration. Supports single keys (`A`, `F1`, `Delete`, etc.) and key combinations
(`Ctrl+C`, `Super+Shift+A`, etc.).

##### Supported Modifiers

| Modifier | Description                       |
|----------|-----------------------------------|
| `Super`  | Super key (Windows/Command key)   |
| `Shift`  | Shift key                         |
| `Ctrl`   | Ctrl key                          |
| `Alt`    | Alt key                           |

##### Binding Types

- **Action binding:** Define actions directly inside the key block; executed when pressed. Multiple actions within the
  same key block are executed in order as written.
- **Group binding:** Key block contains child key bindings instead of actions (if actions are present, actions binding
  is ignored). Pressing the key enters a sub-binding page. The group description is automatically prefixed with `+` to
  indicate it is a group.

##### Action Types

| Action                              | Description                         |
|-------------------------------------|-------------------------------------|
| `spawn "program" "arg1" "arg2" ...` | Launch a program                    |
| `sh "shell command"`                | Execute a shell command via `sh -c` |

## Built-in Navigation

The following navigation hints are always shown at the bottom of the panel:

| Key      | Function              |
|----------|-----------------------|
| `Esc`    | go back / close panel |
| `Ctrl+U` | page up               |
| `Ctrl+D` | page down             |

> [!WARNING]
> `Esc`, `Ctrl+U`, `Ctrl+D` are currently hardcoded and cannot be changed via the configuration file. The config file's
> bind section skips binding these three keys.

## D-Bus Interface

The program communicates over the D-Bus session bus:

- **Interface name:** `com.hrtius.WhichKey`
- **Object path:** `/com/hrtius/WhichKey`

| Method | Description          |
|--------|----------------------|
| `Show` | show/redraw panel    |
| `Quit` | quit the daemon      |

## License

[MIT](./LICENSE)
