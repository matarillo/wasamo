---
title: Human-visible GUI smoke
status: live
created: 2026-05-11
---

# Human-visible GUI smoke

この文書は、phase-end retrospective で必要になった場合に実施する
human-visible GUI smoke の手順をまとめる。シェルは `pwsh` を前提にする。

## 目的

`cargo test --workspace` は headless / automated な検証を担う。一方で、
window が実際に表示されるか、ボタンをクリックして表示テキストが更新されるか、
ウィンドウを閉じてもクラッシュしないかは、人間が見て操作する必要がある。

この smoke は、その human-visible な最小確認である。

## 必要になる条件

phase-end で、次のようなユーザー可視の挙動に影響しうる変更を含む場合は
実施する。

- `wasamo-runtime` の window / layout / text / reactive / ABI / IR loader
- `wasamoc` の parsing / checking / lowering / emit
- `bindings/c`, `bindings/rust`, `bindings/rust-sys`, `bindings/zig`
- `examples/counter-*` または `examples/counter/counter.ui`
- phase の acceptance criteria が「表示される」「クリックで更新される」
  「binding が動く」などの可視挙動を含む場合

docs-only、ADR / plan / retrospective のみ、CI metadata のみなど、
ユーザー可視挙動に影響しない phase では不要としてよい。

## 実行環境

- visible Windows desktop session が必要。
- local physical machine、または画面を見られる RDP / VNC session で実施する。
- plain SSH session は GUI smoke の根拠にしない。
- Visual Studio 2022 Build Tools、Rust stable MSVC target が必要。
- `counter-c` には CMake が必要。
- `counter-zig` には Zig が必要。

## 事前ビルド

repo root で実行する。

```powershell
cargo fmt
cargo clean
cargo build --workspace
cargo build --release --workspace
cargo test --workspace
```

実行時に `wasamo.dll` を見つけられるよう、この `pwsh` session の `PATH` に
release directory を追加する。

```powershell
$env:PATH = "$PWD\target\release;$env:PATH"
```

## 期待結果

各 counter example で以下を確認する。

- 800 x 600 の window が開く。
- window title は現状 `"Wasamo"` でよい。
- `"Count: 0"` が表示される。
- `"Increment"` button をクリックすると `"Count: 1"`, `"Count: 2"` と増える。
- window を閉じても crash しない。

## counter-rust

repo root で実行する。

```powershell
cargo clean -p counter-rust
cargo build --release -p counter-rust
.\target\release\counter-rust.exe
```

詳細は [counter-rust README](../../examples/counter-rust/README.md) を参照。

## counter-c

repo root で実行する。

`cmake` に `PATH` が通っている場合:

```powershell
Remove-Item -Recurse -Force .\build\counter-c -ErrorAction SilentlyContinue
cmake -S examples/counter-c -B build/counter-c
cmake --build build/counter-c --config Release
Copy-Item .\target\release\wasamo.dll .\build\counter-c\Release\
.\build\counter-c\Release\counter.exe
```

Visual Studio 同梱 CMake をフルパスで使う場合:

```powershell
$cmake = "C:\Program Files\Microsoft Visual Studio\18\Community\Common7\IDE\CommonExtensions\Microsoft\CMake\CMake\bin\cmake.exe"

Remove-Item -Recurse -Force .\build\counter-c -ErrorAction SilentlyContinue
& $cmake -S examples/counter-c -B build/counter-c
& $cmake --build build/counter-c --config Release
Copy-Item .\target\release\wasamo.dll .\build\counter-c\Release\
.\build\counter-c\Release\counter.exe
```

MSVC compiler / linker の検出に失敗する場合は、同じ `pwsh` session で
Developer Shell を読み込んでから再実行する。

```powershell
Import-Module "C:\Program Files\Microsoft Visual Studio\18\Community\Common7\Tools\Microsoft.VisualStudio.DevShell.dll"
Enter-VsDevShell -VsInstallPath "C:\Program Files\Microsoft Visual Studio\18\Community" -SkipAutomaticLocation
```

詳細は [counter-c README](../../examples/counter-c/README.md) を参照。

## counter-zig

repo root で実行する。

```powershell
Push-Location .\examples\counter-zig
Remove-Item -Recurse -Force .\.zig-cache, .\zig-out -ErrorAction SilentlyContinue

$zigArgs = @(
  "-Dwasamo-lib=../../target/release/wasamo.dll.lib"
  "-Dwasamo-zig=../../bindings/zig/wasamo.zig"
  "-Dwasamoc=../../target/release/wasamoc.exe"
  "-Doptimize=ReleaseSafe"
)

zig build @zigArgs
Copy-Item ..\..\target\release\wasamo.dll .\zig-out\bin\
.\zig-out\bin\counter-zig.exe

Pop-Location
```

詳細は [counter-zig README](../../examples/counter-zig/README.md) を参照。

## 記録例

phase-end retrospective には、必要/不要の判定と結果を短く記録する。

```md
human-visible GUI smoke: 必要、green。
- counter-rust: window opens, Count: 0 displayed, Increment updates Count: N, close succeeds.
- counter-c: same behavior confirmed.
- counter-zig: same behavior confirmed.
- Note: window title remains the current default "Wasamo"; DSL title is known to be dropped for now.
```
