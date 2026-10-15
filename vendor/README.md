# vendor

本地补丁依赖，用于消除第三方 crate 的 future-incompat 警告。

| 目录 | 说明 |
|------|------|
| `proc-macro-error2` | 将 `extern crate proc_macro` 改为 `pub`，修复 `pub_use_of_private_extern_crate`（与 [proc-macro-error3](https://github.com/gamma0987/proc-macro-error3) / rust#127909 相同）。经根 `Cargo.toml` 的 `[patch.crates-io]` 覆盖 crates.io 版本。 |

待 Leptos 等上游迁到 `proc-macro-error3` 后可删除此目录与 patch。
