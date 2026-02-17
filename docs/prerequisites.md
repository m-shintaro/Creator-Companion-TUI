# 前提条件 / Prerequisites（macOS / Unity 2022.3.22f1）

このツールは VRChat VPM ワークフローのために `vpm` CLI を呼び出します。
Unity でのビルド/アップロードは Unity 側の準備（Unity Hub + 対応 Unity + Build Support）が別途必要です。

This tool calls the `vpm` CLI for VRChat VPM workflows.
Building/uploading in Unity requires separate setup (Unity Hub + supported Unity + Build Support).

---

## 1) .NET 8 SDK（必須：vpm CLI の前提）

公式ページから macOS の **SDK** を選んでインストール:
https://dotnet.microsoft.com/download/dotnet/8.0

```bash
# 確認
dotnet --version
```

`.NET 8 SDK` が必要であることは [VPM CLI 公式ドキュメント](https://vcc.docs.vrchat.com/vpm/cli/) に記載されています。

---

## 2) VCC（CLI）= vpm CLI（必須：PATH に `vpm` が必要）

[VPM CLI 公式（インストール/更新/削除）](https://vcc.docs.vrchat.com/vpm/cli/)

### インストール

```bash
dotnet tool install --global vrchat.vpm.cli
```

### 動作確認

```bash
vpm --version
```

### テンプレート導入（新規作成/移行でほぼ必須）

```bash
vpm install templates
```

テンプレートの保存先（macOS）: `~/.local/share/VRChatCreatorCompanion/VRCTemplates`

### 更新

```bash
dotnet tool update --global vrchat.vpm.cli
```

### 削除

```bash
dotnet tool uninstall --global vrchat.vpm.cli
```

---

## 3) Unity Hub（実質必須：Editor とモジュール管理）

1. [Unity Hub をダウンロード](https://unity.com/download)
2. インストールして起動

---

## 4) Unity Editor（必須：VRChat 推奨 = 2022.3.22f1）

VRChat が現在使用している Unity は **2022.3.22f1** です。
[Current Unity Version（公式）](https://creators.vrchat.com/sdk/upgrade/current-unity-version/)

### インストール

上の「Current Unity Version」ページを開き、本文中の **`2022.3.22f1`** のリンクをクリック
→ Unity Hub が開き、該当バージョンのインストール画面になります（Install / Continue を押すだけ）。

参考: [Unity 2022.3.22f1 リリースノート](https://unity.com/releases/editor/whats-new/2022.3.22f1)

---

## 5) （重要）Windows が選べない時：Unity Hub で Build Support を追加

macOS で Build Settings の Target Platform が macOS しか出ない場合、**Build Support モジュール未導入**が原因です。

### 追加手順

1. Unity Hub → **Installs**
2. `2022.3.22f1` の右側 **…（三点）** を押す
3. **Add Modules** を押す
4. 目的に合わせてチェックしてインストール:
   - Windows 向け: **Windows Build Support**（IL2CPP / Mono など）
   - Android 向け: **Android Build Support**（必要一式）

参考:
- [Build Settings（Unity 公式マニュアル）](https://docs.unity3d.com/2023.2/Documentation/Manual/BuildSettings.html)
- [Add Modules 手順（Unity 公式）](https://docs.unity3d.com/2020.1/Documentation/Manual/GettingStartedAddingEditorComponents.html)

---

## 最短チェックコマンド

```bash
# 1) dotnet
dotnet --version

# 2) vpm
vpm --version

# 3) vpm テンプレート一覧
vpm list templates
```
