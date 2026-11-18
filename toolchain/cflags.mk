# boring-sys 的 bindgen 在交叉编译时用 --target=aarch64-unknown-linux-musl 调 clang，
# 但 clang 看到的是宿主 /usr/include（glibc 头），拼不到 musl 的 bits/。
# 解决：把 zig 自带的 musl 头目录显式喂给 clang。
ZIG_LIB=/usr/lib/zig0.16/lib
BORING_C_INCLUDE_DIRS = -isystem $(ZIG_LIB)/libc/include/generic-musl -isystem $(ZIG_LIB)/libc/include/aarch64-linux-musl -isystem $(ZIG_LIB)/libc/include/any-linux-any
