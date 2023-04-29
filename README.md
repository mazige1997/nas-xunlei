# nas-xunlei

nas-xunlei从迅雷群晖套件中提取，用于发行版Linux（支持OpenWrt）的迅雷远程下载程序。仅供测试，测试完请大家自觉删除。

- 只支持**X86_64**/**aarch64**
- 内侧邀请码（3H9F7Y6D）

```shell
❯ ./xunlei                   
Synology Nas Thunder runs on Linux

Usage: xunlei [OPTIONS] <COMMAND>

Commands:
  install    Install xunlei
  uninstall  Uninstall xunlei
  launch     Launch xunlei
  help       Print this message or the help of the given subcommand(s)

Options:
  -d, --debug    Enable debug mode
  -h, --help     Print help
  -V, --version  Print version

❯ ./xunlei install --help
Install xunlei

Usage: xunlei install [OPTIONS]

Options:
  -d, --debug                          Enable debug mode
  -i, --internal                       Xunlei internal mode
  -p, --port <PORT>                    Xunlei web-ui port [default: 5055]
  -c, --config-path <CONFIG_PATH>      Xunlei config directory [default: /var/packages/pan-xunlei-com]
  -d, --download-path <DOWNLOAD_PATH>  Xunlei download directory [default: /tmp/downloads]
  -h, --help                           Print help

```

### 自行编译

```shell
git clone https://github.com/gngpp/nas-xunlei && cd nas-xunlei

# 默认编译在线安装
cargo build --release && mv target/release/xunlei .

# 完整打包编译安装
bash +x ./unpack.sh && cargo build --release --features embed && mv target/release/xunlei .

# 执行安装
./xunlei install
# 若系统不支持systemctl，则手动启动daemon
./xunlei launch
```
