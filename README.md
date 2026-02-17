# Creator-Companion-TUI (CCT)

> **English?** → [Jump to English section](#english)

VRChat VPM ワークフロー用ターミナル UI ツール。
VCC（VRChat Creator Companion）のナビゲーションモデルを参考にした macOS 向け TUI です。

## 前提条件

- macOS
- Rust ツールチェーン（`rustc 1.80.x` で検証済み）
- `vpm` CLI が `PATH` に存在すること

.NET 8 SDK・vpm CLI・Unity Hub・Unity Editor のインストール手順は **[docs/prerequisites.md](docs/prerequisites.md)** を参照してください。

## ビルド・実行

```bash
cargo build
cargo run
```

テスト:

```bash
cargo test
```

## 画面構成

| 画面 | 説明 |
|------|------|
| **New** | テンプレートからプロジェクト作成（Avatar / World / UdonSharp） |
| **Add** | 既存プロジェクトの登録（単体 / フォルダ一括スキャン） |
| **Projects** | プロジェクト一覧・選択・検索 |
| **Manage** | プロジェクト単位のパッケージ管理 |
| **Settings** | リポジトリ追加・環境チェック・vpm コマンド |

起動時のデフォルト画面は **Projects** です。

## キーバインド

### 共通

| キー | 動作 |
|------|------|
| `q` | 終了 |
| `Tab` / `→` / `←` | 画面切り替え |
| `↑` / `↓` | ログペインのスクロール |
| `Ctrl-C` | 終了 |

### New

| キー | 動作 |
|------|------|
| `j` / `k` | テンプレート選択 |
| `n` | 作成入力を開始 |

入力モード中: `Tab` でフィールド切替（name / path）、`Enter` で実行、`Esc` でキャンセル。

### Add

| キー | 動作 |
|------|------|
| `a` | プロジェクトパスを手動入力して追加 |
| `f` | 親フォルダを指定して一括スキャン（1階層） |

### Projects

| キー | 動作 |
|------|------|
| `/` | 検索 |
| `j` / `k` | プロジェクト選択 |
| `a` | Add 画面へ移動 |
| `Enter` | 選択プロジェクトの Manage 画面を開く |

### Manage

2ペイン構成: Available（利用可能パッケージ）/ Installed（インストール済み）。

| キー | 動作 |
|------|------|
| `h` / `l` | Installed / Available ペインにフォーカス切替 |
| `j` / `k` | フォーカス中のペインで選択移動 |
| `+` | Available ペインで選択中のパッケージをインストール（`=` `:` `＋` `a` も可） |
| `-` | Available ペインで選択中のパッケージを削除（`_` `－` `x` も可） |
| `d` / `D` | Installed ペインで選択中のパッケージを削除 / 強制削除 |
| `u` | Installed ペインで選択中のパッケージを最新バージョンに更新 |
| `U` | VRChat SDK パッケージを最新に更新 |
| `i` | パッケージ名を直接入力してインストール |
| `v` | `vpm resolve project` を実行 |
| `/` | Available パッケージの検索・フィルタ |
| `r` | マニフェスト再読み込み |
| `R` | 利用可能パッケージカタログを VCC キャッシュから再読み込み |

### Settings

| キー | 動作 |
|------|------|
| `1` | nadena リポジトリ追加 |
| `2` | lilToon リポジトリ追加 |
| `a` | カスタムリポジトリ URL を入力して追加 |
| `r` | `vpm list repos` |
| `t` | `vpm install templates` |
| `h` | `vpm check hub` |
| `u` | `vpm check unity` |
| `l` | `vpm list unity` |
| `s` | `vpm open settingsFolder` |
| `c` | 最新の実行中タスクをキャンセル |

## データ保存先

- 設定: `~/.config/vcc-tui/config.json`
- キャッシュ: `~/.cache/vcc-tui/`
- 利用可能パッケージの読み込み元: `~/.local/share/VRChatCreatorCompanion/Repos/`

---

# English

A terminal UI companion for managing VRChat creator projects and VPM packages.
macOS-focused TUI with a VCC-like navigation model.

## Prerequisites

- macOS
- Rust toolchain (validated with `rustc 1.80.x`)
- `vpm` CLI available in `PATH`

For installation steps (.NET 8 SDK, vpm CLI, Unity Hub, Unity Editor), see **[docs/prerequisites.md](docs/prerequisites.md)**.

## Build / Run

```bash
cargo build
cargo run
```

Test:

```bash
cargo test
```

## Screens

| Screen | Description |
|--------|-------------|
| **New** | Create project from template (Avatar / World / UdonSharp) |
| **Add** | Import existing projects (single path / one-level folder scan) |
| **Projects** | Project list, selection, and search |
| **Manage** | Per-project package operations |
| **Settings** | Repo management, environment checks, vpm commands |

Default screen on startup is **Projects**.

## Keybindings

### Global

| Key | Action |
|-----|--------|
| `q` | Quit |
| `Tab` / `→` / `←` | Navigate screens |
| `↑` / `↓` | Scroll log pane |
| `Ctrl-C` | Quit |

### New

| Key | Action |
|-----|--------|
| `j` / `k` | Select template |
| `n` | Open create input |

In input mode: `Tab` to switch field (name / path), `Enter` to run, `Esc` to cancel.

### Add

| Key | Action |
|-----|--------|
| `a` | Add single project path |
| `f` | Add projects from parent folder (one-level scan) |

### Projects

| Key | Action |
|-----|--------|
| `/` | Search |
| `j` / `k` | Select project |
| `a` | Go to Add screen |
| `Enter` | Open Manage screen for selected project |

### Manage

Two-pane layout: Available packages / Installed packages.

| Key | Action |
|-----|--------|
| `h` / `l` | Focus Installed / Available pane |
| `j` / `k` | Move selection in focused pane |
| `+` | Install selected available package (also `=` `:` `＋` `a`) |
| `-` | Remove selected available package (also `_` `－` `x`) |
| `d` / `D` | Remove / force-remove selected installed package |
| `u` | Update selected installed package to latest version |
| `U` | Update VRChat SDK package to latest |
| `i` | Install package by typing name directly |
| `v` | Run `vpm resolve project` |
| `/` | Search/filter available packages |
| `r` | Reload manifest |
| `R` | Reload available package catalog from VCC cache |

### Settings

| Key | Action |
|-----|--------|
| `1` | Add nadena repo |
| `2` | Add lilToon repo |
| `a` | Add custom repo URL |
| `r` | `vpm list repos` |
| `t` | `vpm install templates` |
| `h` | `vpm check hub` |
| `u` | `vpm check unity` |
| `l` | `vpm list unity` |
| `s` | `vpm open settingsFolder` |
| `c` | Cancel latest running task |

## Data locations

- Config: `~/.config/vcc-tui/config.json`
- Cache: `~/.cache/vcc-tui/`
- Available packages loaded from: `~/.local/share/VRChatCreatorCompanion/Repos/`

## License

MIT
