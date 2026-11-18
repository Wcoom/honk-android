#!/system/bin/sh
# setup-cgroup.sh —— 让根 cgroup 的 BPF hook 能与 honk 共存（手机侧执行，需 root）
#
# 背景：Android 内核在根 cgroup 上预先挂了 15 个 BPF 程序，全部是 legacy 模式
# （attach_flags=0）。内核规则：已有 legacy 程序的 hook，不允许再叠加任何程序
# （BPF_F_ALLOW_MULTI 也挂不上，EPERM）。honk 要往根 cgroup 挂 6 个必需程序，
# 所以必须先把这些 legacy 占用"升级"成 multi 模式。
#
# 做法：detach 再以 multi 重新挂回同一个程序。程序本身不变、功能不变，
# 只是把该 hook 变成可叠加。实测网络、mihomo 均不受影响。
#
# ⚠️ 这只在本 boot 有效，真重启后恢复 legacy，需要重新执行。
#
# 用法：su -c 'sh setup-cgroup.sh'

set -u
CGROUP=/sys/fs/cgroup

upgrade() {
    t=$1; id=$2
    if bpftool cgroup show $CGROUP 2>/dev/null | grep -q "$t .*multi"; then
        echo "  $t 已是 multi，跳过"
        return 0
    fi
    bpftool cgroup detach $CGROUP $t id $id >/dev/null 2>&1
    if bpftool cgroup attach $CGROUP $t id $id multi >/dev/null 2>&1; then
        echo "  $t -> multi OK"
    else
        echo "  $t 升级失败（程序 $id 可能已变，重新查 id 后重试）"
    fi
}

echo "== 升级 honk 需要的 cgroup hook 到 multi 模式 =="
upgrade cgroup_sock_ops              19
upgrade cgroup_inet_sock_create      29
upgrade cgroup_inet4_connect         33
upgrade cgroup_inet6_connect         34
upgrade cgroup_udp4_sendmsg          37
upgrade cgroup_udp6_sendmsg          38
upgrade cgroup_inet4_bind            31
upgrade cgroup_inet6_bind            32
upgrade cgroup_inet_sock_release     30

echo
echo "== 结果 =="
bpftool cgroup show $CGROUP | grep -vE '^ID|^Cgroup'
