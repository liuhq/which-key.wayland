# which-key.wayland

一个 Wayland 环境的按键提示面板，灵感来源于 Neovim 插件 [which-key.nvim](https://github.com/folke/which-key.nvim) 和
[Helix](https://helix-editor.com/) 编辑器风格。

`which-key.wayland` 可以在支持 `wlr_layer_shell` 协议的 Wayland
合成器上显示一个按键提示面板，通过按键序列浏览和执行命令。

## 安装

### Cargo

```sh
cargo install which-key-wayland
```

### Nix Flake

#### :snowflake: 快速试用

如果你已安装 Nix 或正在使用 NixOS，可直接运行：

```sh
nix run github:liuhq/which-key.wayland
```

#### NixOS 安装

在 `flake.nix` 中引入并安装：

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

### 从源码编译

编译环境要求：

- Rust 工具链 (`rustc`、`cargo`)
- `libxkbcommon`、`wayland` (运行时与构建依赖)
- `pkg-config` (构建工具)

```sh
# clone
git clone https://github.com/liuhq/which-key.wayland.git
cd which-key.wayland
# build
cargo build --release
# run
./target/release/which-key-wayland
```

## 使用方式

程序会在后台常驻运行，通过 D-Bus 会话通信。

```sh
which-key-wayland       # 启动程序并显示面板 (若已运行则自动唤起面板)
which-key-wayland show  # 向运行中的实例发送唤起命令
which-key-wayland quit  # 退出程序
```

推荐在窗口管理器/混成器中绑定快捷键到 `which-key-wayland`，程序会自动处理首次启动与后续唤起。

示例：

**Niri** (`$XDG_CONFIG_HOME/niri/config.kdl`)：

```kdl
binds {
    Mod+Space { spawn "which-key-wayland"; }
}
```

**Hyprland** (`$XDG_CONFIG_HOME/hypr/hyprland.conf`)：

```text
bind = $mainMod, Space, exec, which-key-wayland
```

## 配置文件

配置文件使用 [KDL](https://kdl.dev/) 格式。

### 优先级

配置文件会按以下优先级读取：

1. `$WKW_CONFIG_FILE` 环境变量指定的路径
2. `$XDG_CONFIG_HOME/which-key-wayland/config.kdl`
3. `$HOME/.config/which-key-wayland/config.kdl` (`XDG_CONFIG_HOME` 未设置时的回退)
4. 以上均不存在或解析失败，使用默认值

### 配置示例

参见 [`examples/config.kdl`](../examples/config.kdl)。

### 配置项说明

#### `timeout`

面板在无操作时自动隐藏的超时时间 (毫秒)。设为 `0` 则永不自动隐藏。

| 字段      | 类型 | 默认值 | 说明           |
|-----------|------|--------|----------------|
| `timeout` | 整数 | `2000` | 毫秒，`0` 禁用 |

#### `font` 块

| 字段          | 类型   | 默认值 | 说明 |
|---------------|--------|--------|------|
| `size`        | 浮点数 | `16.0` | 字号 |
| `line-height` | 浮点数 | `20.0` | 行高 |

#### `color` 块

颜色值支持 `"#RGB"`、`"#RRGGBB"`、`"#RRGGBBAA"` 三种十六进制格式 (`#`
可选)，末尾两位为透明度 (alpha)，省略时默认不透明。

| 字段           | 类型   | 默认值      | 说明                  |
|----------------|--------|-------------|-----------------------|
| `fg-key`       | 字符串 | `"#D8DEE9"` | 按键文字颜色          |
| `fg-separator` | 字符串 | `"#4C566A"` | 分隔符颜色            |
| `fg-action`    | 字符串 | `"#88C0D0"` | `Action` 描述文字颜色 |
| `fg-group`     | 字符串 | `"#5E81AC"` | `Group` 描述文字颜色  |
| `bg`           | 字符串 | `"#2E3440"` | 背景颜色              |

#### `layout` 块

| 字段        | 类型 | 默认值 | 说明                                             |
|-------------|------|--------|--------------------------------------------------|
| `width`     | 整数 | `500`  | 面板宽度 (像素)                                  |
| `max-items` | 整数 | `10`   | 每页显示的最大条目数                             |
| `padding`   | 整数 | `4`    | 内边距 (像素)                                    |
| `radius`    | 整数 | `0`    | 圆角半径 (像素)                                  |
| `anchor`    | 整数 | `2`    | 屏幕锚点：`1` 右上、`2` 右下、`3` 左下、`4` 左上 |

##### `margin` 子块

| 字段     | 类型 | 默认值 | 说明   |
|----------|------|--------|--------|
| `top`    | 整数 | `0`    | 上边距 |
| `right`  | 整数 | `0`    | 右边距 |
| `bottom` | 整数 | `0`    | 下边距 |
| `left`   | 整数 | `0`    | 左边距 |

#### `bind` 块

按键绑定配置。支持单键 (`a`、`F1`、`Delete` 等) 和组合键
(`Ctrl+C`、`Super+a` 等)。

> [!NOTE]
> 按键绑定**区分大小写**：按下 `a` 键匹配配置中的 `a`，而非 `A`。 对于可打印的单字符键，`Shift` 修饰键是隐式的——配置中的
> `Ctrl+c` 匹配键盘上按住 `Ctrl` 再按 `c` 键。

##### 支持的修饰键：

| 修饰键  | 说明                             |
|---------|----------------------------------|
| `Super` | Super 键 (即 Windows/Command 键) |
| `Shift` | Shift 键                         |
| `Ctrl`  | Ctrl 键                          |
| `Alt`   | Alt 键                           |

##### 绑定类型：

- **动作绑定：**
  在按键块内直接定义动作，按下后执行；同一按键块内可包含多个动作，将按书写顺序依次执行
- **分组绑定：**
  按键块内包含子按键而非动作 (若包含动作，则忽略动作绑定)，按下后进入子绑定页面；分组描述会自动添加 `+` 前缀以标示为分组

##### 动作类型：

| 动作                               | 说明                         |
|------------------------------------|------------------------------|
| `spawn "程序" "参数1" "参数2" ...` | 启动一个程序                 |
| `sh "shell命令"`                   | 通过 `sh -c` 执行 shell 命令 |

## 内置导航

面板底部固定显示导航提示：

| 按键     | 功能                  |
|----------|-----------------------|
| `Esc`    | 返回上一级 / 退出面板 |
| `Ctrl+u` | 向上翻页              |
| `Ctrl+d` | 向下翻页              |

> [!WARNING]
> `Esc`、`Ctrl+u`、`Ctrl+d` 当前为硬编码快捷键，尚不支持通过配置文件修改。配置文件中 bind 会跳过这三个键的绑定。

## D-Bus 接口

程序通过 D-Bus 会话总线进行进程间通信：

- **接口名：** `com.hrtius.WhichKey`
- **对象路径：** `/com/hrtius/WhichKey`

| 方法   | 说明          |
|--------|---------------|
| `Show` | 显示/重绘面板 |
| `Quit` | 退出守护进程  |

## 许可证

[MIT](../LICENSE)
