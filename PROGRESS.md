# honk 移植到 Android —— 项目状态与进度

> 最后更新：2026-11-21
> 目标设备：OnePlus 13（PJZ110），Android 16，内核 **6.6.118** GKI，
> bootloader 已回锁，root 来自 KernelSU **LKM** 模式。

---

## 1. 项目目标与当前结论

把 [daeuniverse/honk](https://github.com/daeuniverse/honk)（Rust + eBPF 透明代理）
移植到 Android 6.6 GKI 内核上，并对接 [boxproxy/boxproxy](https://github.com/boxproxy/boxproxy)。

**当前状态：✅ 全链路打通并真机实测通过**
- **8 个 TC 分类器变体**（`wan_egress_l2/l3`、`wan_ingress_l2/l3`、`lan_ingress_l2/l3`、`lan_egress_l2/l3`）以及 `dae0_ingress` / `dae0peer_ingress` / `tproxy_sk_lookup` / 6 个 cgroup 程序 + freplace 路由扩展 **全部通过 Android GKI 6.6.118 verifier 并成功挂载**。
- **Android 路由与网络栈适配完成**：解决 Android netd 策略路由 `fwmark` 冲突（`0x100` → `0x10000000`）与 `wan_interface: auto` 在无 main 表默认路由时的自动探测（解析 `/proc/net/fib_trie` 中的 `1000 + ifindex` 策略表）。
- **零环境变量自包含启动**：`honk-core` 启动时自动通过 `BPF_PROG_QUERY` + `BPF_PROG_GET_FD_BY_ID` 将根 cgroup 上的 legacy BPF 钩子无损升级为 `BPF_F_ALLOW_MULTI`，自动探测 `/apex/com.android.conscrypt/cacerts`，自动回退锁目录与数据目录，并兼容 BoxProxy `-t / -d / -f` 参数。
- **端到端流量实测通过**：手机上直连（`www.baidu.com` → HTTP 200）与 AnyTLS 节点代理（`1.1.1.1/cdn-cgi/trace` → `colo=NRT, loc=JP`、`www.google.com/generate_204` → HTTP 204）以及 `boxctl --core honk --mode ebpf up/status/down` 生命周期管理全部验证通过。

---

## 2. 目录结构

```
honk-android/
├── src/                  honk 源码（基于 v0.1.0-beta.6 / 8a7611e）+ Android 适配补丁
├── toolchain/
│   ├── aarch64-linux-musl-gcc    cc-rs 找的 C 编译器（zig 冒充 musl gcc）
│   ├── aarch64-linux-musl-g++    同上（C++）
│   └── zig-link                  rustc 用的链接器包装器
├── lib/                  zig 预构建的 aarch64 运行时归档
│   ├── libc++.a / libc++abi.a    boring-sys 需要（musl 无 libstdc++）
│   ├── libubsan_rt.a             aws-lc-sys 的 jitterentropy 带 UBSan 插桩
│   └── libcompiler_rt.a          上面两者依赖的 __extendxftf2 等
├── build.sh              一键交叉编译 → aarch64 静态 honk-core
├── build-boxctl.sh       一键交叉编译 → aarch64 静态 boxctl
├── vtest.sh              快速编译 eBPF 对象并在真机验证 verifier
├── stackmap.py           静态分析 eBPF 调用链栈深与指令数的辅助脚本
├── setup-cgroup.sh       手机侧手工升级根 cgroup legacy hook 的备用脚本（已内置进 honk-core）
├── boxproxy/             boxproxy 源码（已适配 `--core honk --mode ebpf`）
├── config/               脱敏后的 Android 推荐配置模板（`config.example.dae`）
├── patches/              基于上游基线导出的完整补丁（`honk-android.patch` / `boxctl-honk.patch`）
├── skill/honk-android/   供 AI 助手自动完成部署、迁移与验证的标准化 Skill
├── README.md             仓库总览、构建与快速部署说明
├── GUIDE.md              人类零基础部署与双面板使用指南
├── CHANGELOG.md          版本变更记录
├── PATCHES.md            对上游 honk 的改动清单
├── ANDROID-DATAPATH.md   6.6 verifier 卡点根因与解法分析
└── PROGRESS.md           本文件
```

---

## 3. 核心问题攻克记录

### 3.1 攻克 6.6 eBPF Verifier（1M 指令上限 + 512B 栈上限）

详见 `ANDROID-DATAPATH.md`。6.6 verifier 失败由两个机制叠加导致：
1. **平级子程序分派（解决指令数 + 栈深度冲突）**：
   - 将 `do_tproxy_wan_egress` 拆为三个**平级** `#[inline(never)]` 子程序：`wan_egress_prelude`（64B 栈）、`do_tproxy_wan_egress_tcp_entry`（312B 栈）、`do_tproxy_wan_egress_udp_entry`（296B 栈），由入口 `wan_egress_dispatch` 顺序分派（前置返回 `WAN_CONTINUE_TCP` / `WAN_CONTINUE_UDP`）。
   - 子程序统一返回扁平 `c_long`（单寄存器 `r0`），避免返回 `Result<c_long, c_long>` 使用 `r2` 触发 BPF 调用约定 `R2 !read_ok`。
   - 将 `CONNTRACK_SNAPSHOT_MAP` 扩为与 `FlowMapValue<ConnState>` 同布局的 `ConnSnapshotSlot`（80B），使 TCP/UDP `CONN_STATE_MAP.insert` 直接复用 per-CPU map 内存，从 `__mark_tcp_seen` / `__mark_udp_seen` 栈上省下 64B，使最长调用链栈深压到 `320 + 80 + 32 = 432 <= 512`。
2. **彻底消除 `aya_ebpf::memset` 逐字节循环（解决 `wan_egress_l3` 等状态爆炸）**：
   - 原因：`wan_egress_l3` 在进入子程序前调用 `l3_link_h_len()`，使 `link_h_len` 产生 `{0, 14}` 两条状态分支；而 `aya-ebpf` 内置的 `memset` 是逐字节循环（`while i < n`），6.6 verifier 对每次循环迭代追踪精确标量 `r4=0..N`，任何残余的 `memset(72)` / `memset(100)` 在多分支下都会把状态数乘以 N，直接冲破 1M 指令。
   - 解法：在 `honk-ebpf-common::conn` 实现按对齐字宽展开的 `zero_in_place_volatile<T>`（8 字节对齐结构体如 `ConnState` 72B 仅需 9 条 `u64` volatile store，零循环、零分支、零栈临时量），替换 `ParseTransportCtx`、`Tuples`、`ConnState`、`ConntrackArgs`、`RedirectEntry`、`RoutingInput`、`DaeEvent` 等所有大结构体清零。

### 3.2 攻克 Android netd 策略路由与 `wan_interface: auto`

1. **`DAE_BYPASS_MARK` 与 Android `ip rule` 冲突**：
   - Android `ip rule` 使用 `fwmark` 低 16 位存放 `netId`，兜底规则 `31000: from all fwmark 0x0/0xffff iif lo lookup 1021` 要求低 16 位必须为 0，否则落入 `32000: unreachable`。
   - 上游默认 `DAE_BYPASS_MARK = 0x100`（低 16 位非零），导致 honk 自身发出的所有直连与上游 DNS 请求直接报 `ENETUNREACH (os error 101)`。
   - 解法：将 `DAE_BYPASS_MARK` 与 `GlobalConfig::effective_so_mark()` 默认值改为 `0x1000_0000`（Bit 28，避开低 16 位 netId、避开 netd 高位策略位、也避开 honk 保留位 `0xc800_0000`）。
2. **`wan_interface: auto` 默认路由接口探测**：
   - Android 的 main 表（`/proc/net/route`）没有任何 `0.0.0.0/0` 默认路由，真实默认路由在 netd 的 per-network 策略表 `1000 + ifindex`（如 `rmnet_data2` ifindex=21 → table `1021`）。
   - 解法：当 `/proc/net/route` 无默认路由时，解析 `/proc/thread-self/net/fib_trie` 中 `Id <1000+ifindex>:` 下的 `0.0.0.0/0 universe UNICAST` 路由反推 `ifindex` 并映射为接口名（同时保留 UDP socket connect + `getifaddrs` 作为二级兜底）。

### 3.3 Android 自包含运行与 BoxProxy 集成

1. **自动升级根 cgroup legacy BPF 钩子**：
   - `honk-core` 在 `RealEbpfBackend::load` 挂载 cgroup 程序前，自动对 9 个相关 cgroup attach type 执行 `BPF_PROG_QUERY`；若检测到 `attach_flags == 0`（legacy 模式），自动通过 `BPF_PROG_GET_FD_BY_ID` 拿到原程序 FD，`BPF_PROG_DETACH` 后立刻以 `BPF_F_ALLOW_MULTI` 重新 `BPF_PROG_ATTACH` 同一个程序。开机后无需手工跑 `setup-cgroup.sh`。
2. **零环境变量依赖**：
   - 自动检测 `/apex/com.android.conscrypt/cacerts` 或 `/system/etc/security/cacerts` 并设置 `SSL_CERT_DIR`（若外部誤设 `SSL_CERT_FILE` 为目录则自动纠正，消除 `Is a directory` 警告）。
   - 无 `/run` 时锁文件自动回退到 `/data/local/tmp/honk-core.lock`，无 `/var/run` 时静默跳过 `ip netns` 兼容挂载，无 `/var/lib/honk` 时数据目录自动回退到 `-d` 参数或配置文件同级 `data/` 目录。
3. **BoxProxy (`boxctl`) 对接**：
   - `honk-core` 支持 `-t`（测试配置并退出）、`-d <workdir>`、`-f <config.dae>`（兼容 mihomo 风格参数）。
   - `boxproxy/boxctl` 新增 `honk` 核心支持（`--core honk --mode ebpf`），并向后兼容手机上现有 `box.db` schema。
   - 已将 honk 二进制部署至手机 `/data/user/0/com.boxproxy.box/files/box/bin/honk`。

### 3.4 生产部署与 Bug 测试（2026-11-19）

1. **生产配置部署（参考手机 mihomo 配置）**：
   - 将手机原 `mihomo/config.yaml` + `proxy3` 节点完整迁移至 `/data/user/0/com.boxproxy.box/files/box/honk/config.dae`（126 个有效代理节点：2 个 VLESS-XHTTP + 124 个 AnyTLS，已剔除流量信息行与 `9.9.9.9:1` 占位节点；48 个策略组与 53 条路由规则，完整保留 `总模式` / 地区 `score` / `延迟最低` / `故障转移` / `负载均衡` / 应用分流 / Play CDN 直连 / QUIC UDP 443 拦截 / `WebRTC` / `Emby` / `Gemini` / `PikPak` 等规则）。
   - 部署 `geoip.dat` 与 `geosite.dat` 至 `/data/user/0/com.boxproxy.box/files/box/honk/data/`（`root:net_admin`, `0700`），使 `honk.db` 状态持久化与地理规则均正常工作。
   - 对接 Clash API（`0.0.0.0:9090`）与现有 Zashboard UI（`/data/user/0/com.boxproxy.box/files/box/mihomo/Web/Zash`）。
   - 备份原版 `boxctl` 与 `box.db` 至 `/data/user/0/com.boxproxy.box/files/box/bin/meta-backup/{boxctl.orig,box.db.orig}`，安装新版 `boxctl` 并将 `box.db` 的 `runtime_profile` 切换为 `core_name='honk', mode='ebpf', config_name='config.dae'`（兼容 `/data/adb/service.d/boxproxy-box.sh` 开机自启与 `boxctl up/down/restart/status`）。
2. **Bug 测试与修复记录**：
   - **修复 DNS 上游语法 & 失效 DoH**：原 mihomo 的 2 个国内自定义 DoH 在当前移动网络直连超时，改为国内可用 DoH（`dns.alidns.com` / `doh.pub`）与 UDP `223.5.5.5:53`；将 `-> direct` / `-> 总模式` 置于单引号外部，消除 `legacy-upstream-separator` 与运行时 host 解析报错。
   - **修复 `geosite:google-cn` 污染 `www.gstatic.com` DNS 解析**：移除 DNS 请求分流中的 `geosite:google-cn` 并将 `google.com` / `googleapis.com` / `googleusercontent.com` / `gstatic.com` / `ggpht.com` 显式路由至 `remote_doh`，修复前 `www.gstatic.com` 解析为国内 IP `223.106.89.226`，修复后经代理 DoH 正常返回海外 Google IP（如 `203.208.49.130` / `142.251.157.120`），同时保留 `redirector.gvt1.com` 走国内 `120.253.253.161` 直连。
   - **修复 `手动选择` 组默认节点名失效**：原 mihomo 缓存记录的 `🇸🇬 Singapore 02` 与实际节点名 `🇸🇬 Singapore 02 [DIP Singapore]` 不一致，导致 Clash API 返回 `proxy not found`，已更正并验证延迟测速通过（322ms）。
   - **修复 `boxctl` 对 `honk` 启动探测窗口过短**：`boxproxy/boxctl/src/service.rs` 原 `STARTUP_PROBE_DELAY_MS = 300ms` 短于 `honk` 加载 eBPF+DNS 所需的 ~0.9s，已为 `honk` 改为 1500ms 轮询探测（`cargo +1.99.1 test` 35/35 通过），确保 `boxctl up/restart` 返回时 `honk` 与 Clash API 已完全就绪且能捕获启动期异常。
   - **Android 15 字节 `pname` 前缀适配验证**：通过实机 `/connections` 观测证实 `honk` eBPF `argv[0]` 捕获截断为 15 字节（如 `com.oplus.dmp:p`），配置中已对长包名使用精确 15 字节前缀（如 `com.pikcloud.pi`、`com.openai.chat`、`com.instagram.a`）。
   - **已知上游源站异常排查**：部分测试节点经宿主 PC 直接 `curl --http2` 亦超时无响应，确认系该节点远端源站不可用，非 `honk` XHTTP 客户端问题。
3. **验证通过项**：
   - 进程与内核卫生：`boxctl restart` 后 `honk` 正常运行，无重复/泄漏 `dae0` 或 BPF cgroup 钩子（`bpftool cgroup list` 精确 6 个 honk 程序），`dmesg` 无内核/BPF 异常，FD 约 161–187，RSS 约 54–78 MB。
   - Clash API 与 Zashboard：`/version`、`/configs`、`/proxies`、`/rules`、`/connections`、`/ui/` 与 `/ui/icons/Main.png` 均返回 200；策略组选择经 `honk.db` 跨 `boxctl restart` 持久化通过。
   - 数据面验证：国内直连（`www.baidu.com` 200、`myip.ipip.net` 返回本机运营商公网 IP）、海外代理（`1.1.1.1/cdn-cgi/trace` 经海外出口、`www.google.com/generate_204` 204、`www.gstatic.com/generate_204` 204、`speed.cloudflare.com` 5MB 下载 200 约 955 KB/s）、透明 UDP 53 劫持（`busybox nslookup www.google.com 8.8.8.8` 返回干净 IP `142.251.155.119`）、QUIC UDP 443 拦截（1200B QUIC 探测包返回 0 字节）、20 路并发 HTTPS（10 国内 200 + 10 海外 204 全部成功）。
4. **自定义 DNS 与 USB 热点免劫持加固（2026-11-19）**：
   - **持久化管理 CLI**：BoxProxy APK 启动时会覆盖 `$BOX/bin/boxctl` 为上游原版（不含 `honk` 支持），已将支持 `honk` 的版本固定部署至 `$BOX/bin/boxctl-honk`。
   - **自定义加密 DNS（DoH / DoH3 / DoT）**：已将手机 `config.dae` 切换为用户指定的 6 组加密 DNS 上游（2×DoH、2×DoH3/QUIC、2×DoT:853），配合 `bootstrap_resolver: '223.5.5.5:53'` 与 `bootstrap_udp` 规避运营商 DNS 异常解析及循环依赖，实机 debug 日志逐一验证全部 6 个上游在 `honk` 内收发正常；主用 DoH 实测国内/海外解析与 `www.baidu.com`(200)、`www.google.com`(204)、`www.gstatic.com`(204)、`api.github.com` 均通过。
   - **USB 热点免劫持**：确认 `honk` 仅挂接蜂窝/Wi-Fi 广域网接口与 `dae0`（`rndis0` 零 `honk` eBPF 钩子），内核 `wan_egress_l3` 通过 `skb_ingress_ifindex != 0` 自动放行所有 `rndis0`/`ncm0`/`usb0` 转发流量（含 BPF tether offload 与常规 NAT 转发，不依赖动态分配子网）；并在路由顶部新增 `pname('dnsmasq') -> direct(must)` 与 `sip(192.168.42.0/24) -> direct(must)`，彻底消除此前 Android 热点 `dnsmasq`（uid 1052 `dns_tether`）本地发起上游查询被 `honk` 劫持的问题。实测宿主 PC 经 USB 热点访问外网以及向运营商内网 DNS 53 端口发起查询时，`honk` debug 日志拦截计数均为 `0`。
5. **BoxProxy APK 反编译适配 `honk` 核心 + 原生 Doona WebUI 直连（已实机验证）**：
   - **升级 `boxproxy/boxctl`**：兼容新版 BoxProxy APK `26.11.18.9cf5221` CLI 协议（新增 `wifi`/`network-match` 别名、`clashapi` 子命令，运行时配置同步至 `run/state/config/startup-config`，启停自动清理过期 `run/box.identity`）；`cargo +1.99.1 test` 35/35 全通过，静态交叉编译 `aarch64-unknown-linux-musl` 后替换打包进 APK `assets/boxctl/arm64-v8a/boxctl`。
   - **反编译与 Smali 补丁**：使用 `jadx` + `apktool` + `uber-apk-signer` 为 `com.boxproxy.box` 完整接入 `cf5.Honk`（ordinal 5，`.dae`，仅允许 `ebpf` 模式、默认 `config.dae`），覆盖首页状态卡片（`xp2`）、核心/模式选择器（`cf5`/`tc0`/`sd5`/`eo`/`mc3`/`sa1`/`oe1`/`k`）、`box.identity` 校验（`nm`）、Clash API 仪表板解析（`aq0`/`kd0`/`tj5`/`na1`/`re1`/`pb1`，兼容 `external_controller` 下划线写法）、文件白名单（`qn`）以及本地 WebUI 入口（`g9`，`honk` 核心直指 `http://127.0.0.1:9527/ui/`）。
   - **重编 `honk-core` 启用 `native-ui`（内置 Doona 原生面板）**：更新 `build.sh` 在检测到 `HONK_DOONA_DIR`（由 `src/ci/fetch-doona.sh` 拉取，含 `-lite`）时自动启用 `ebpf,rprx,clash-api,native-ui,jemalloc`（否则回退 `native-api`）；在 `config.dae` 中同时开启 `clash_api`（`0.0.0.0:9090`，供 BoxProxy 原生「面板」页使用）与 `native_api`（`127.0.0.1:9527`，`allow_anonymous_loopback: true`，`ui: 'embedded'`，供 BoxProxy「WebUI」直接打开原生 Doona 面板）。
   - **实机验证结果**：
     - BoxProxy 首页正常显示 `运行中`、`Honk · eBPF`、`config.dae`、实时延迟 / WAN IP / 网速 / 内存占用，底部四大标签（首页 / 面板 / 工具 / 设置）0 崩溃；
     - 点击首页 **WebUI（Web 界面）** 直接在内置 WebView 加载原生 **doona** 面板（`http://127.0.0.1:9527/ui/`），实测「活动」与「路由 → 策略（48 组 / 122 节点可用，TCP/UDP 分选状态）」渲染正常、0 崩溃；
     - 代理流量正常（`baidu_cn=200` 直连、`google_proxy=204` 代理）。
6. **22 小时实机长稳压测与 2 处真实缺陷修复（2026-11-20 已实机回归验证）**：
   - **22 小时连续运行表现**：`honk` 单进程连续运行 22 小时转发下行 2.92 GB / 上行 178 MB，`ERROR=0`、`panic=0`、BoxProxy `crash=0`，常驻内存 `62 MB`（峰值 `99 MB`），FD 随活跃连接正常回收，自动完成 5 次蜂窝网卡切换（`rmnet_data10 → 7 → 2 → 4 → 9 → 8`）。
   - **修复 DNS 响应 QNAME 大小写敏感校验 Bug**：`src/crates/honk-core/src/dns/response.rs` 改为 `eq_ignore_ascii_case`（RFC 1035/4343），修复 DoH 上游将大写域名规范化为小写时误判 `QuestionMismatch` 返回 `SERVFAIL` 的问题；新增单元测试（`dns::response::tests` 5/5 通过），实机发送原始 UDP 大写 `Example.COM` 查询经 `honk` eBPF DNS 拦截转发至 DoH 返回 `NOERROR`（`0x8180`，2 条 A 记录），`honk.log` `dns_control` 告警为 `0`。
   - **修复 Android 平台 Bootstrap DNS 单点丢包与回退失效**：`src/crates/honk-outbound/src/bootstrap.rs` 增加 UDP 1 秒自动重传（在 3 秒总预算内）及 `GLOBAL_FALLBACK` 备用直连 DNS 兜底，并在 `honk-core` 启动与热重载路径同步接入 `config.global.fallback_resolver`；新增 2 项单元测试（`bootstrap::tests` 15/15 通过）。
   - **收紧配置与构建脱敏**：手机端 `clash_api.external_controller` 收紧为 `127.0.0.1:9090`（消除 `unsafe-api-bind` 启动告警）；`build.sh` 增加 `--remap-path-prefix` 脱敏构建路径，并将脱敏后的正式版成品（二进制、APK、配置模板、源码补丁、人类使用指南 `GUIDE.md`、AI 自动化 Skill `skill/honk-android/` 及校验和）打包至桌面。
7. **Doona 原生面板全功能开启 + 内核 UDP 跟踪 + QUIC/UDP 全开 + 数据路径健康校验（2026-11-21 已实机验证）**：
   - **ARM64 `kprobe`/`kretprobe` 内核 UDP 接收跟踪**：在 `src/crates/honk-ebpf/src/receive_trace.rs` 与 `src/crates/honk-core/src/ebpf/real/receive_trace.rs` 中新增 `kprobe`/`kretprobe` 回退路径（解决 Android 6.6 GKI 不支持 `fentry`/`fexit` `ENOTSUPP 524` 问题），启动日志 `WARN=0`、`ERROR=0`。
   - **开启完整 UDP / QUIC 转发与 QUIC 节点评分探测**：移除 `l4proto(udp) && dport(443) -> block`，启用 `preconnect_node_count: 2` / `udp_warm_node_count: 2`，首个 `tcp_check_url` 设为 `https://cp.cloudflare.com/generate_204`，消除 `quic_probe_disabled`。
   - **解锁 Doona 全部未开启功能**：新增 `NativeApiConfig::admin_authorized()` 支持环回免密管理授权（`config_write: true`），`boxctl` 启动参数升级为 `-d <dir>/data -c <file> --store db` 并注入 `HONK_RECORD_ALWAYS=1`，同时允许 `--store db` 与本机 `clash_api` 共存，完整开启配置读写、节点/规则/策略组/订阅管理、导入、修订历史、地理数据更新（`assets.geodata`）与常驻后台流程记录。
   - **完整 eBPF 数据路径内核钩子校验**：在 `src/crates/honk-core/src/ebpf/real/{bindings.rs,observation.rs}` 中使用 `BPF_PROG_QUERY` 校验 cgroup、`daens` `sk_lookup` 与 `dae0peer` TCX 钩子，使 `/api/v1/datapath` 状态完整变为 `state="active"`、`hooks="attached"`、`health="healthy"`。
8. **共存版 `BoxProxy Honk` (`com.boxproxy.honk`) 适配与实机双开共存验证（2026-11-21）**：
   - 将适配版 APK 包名改为独立包名 `com.boxproxy.honk`（应用名 `BoxProxy Honk`、磁贴 `Box Honk`），同步隔离签名权限、`androidx-startup` Authority、通知栏广播 Action/Extra、备份前缀与开机自启脚本（`/data/adb/service.d/boxproxy-honk.sh`）。
   - 同步升级 `boxctl` 默认目录探测、`config.example.dae` 与一键部署脚本 `deploy_honk_android.sh`。
   - 真机实测 `com.boxproxy.honk` 与原版 `com.boxproxy.box` 并存安装运行 0 冲突、0 崩溃，迁移后 `honk` 在 `/data/user/0/com.boxproxy.honk/files/box/` 下启动正常（`state="active"`, `hooks="attached"`, `health="healthy"`, `WARN=0`, `ERROR=0`）。
