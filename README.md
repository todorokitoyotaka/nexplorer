# GPTファイルエクスプローラー / GPT File Explorer

![Version](https://img.shields.io/badge/version-0.1.0-blue)
![License](https://img.shields.io/badge/license-MIT-green)

[English](#english) | [日本語](#japanese)

<a name="japanese"></a>
# 日本語

GPTを活用したインテリジェントなファイルエクスプローラーツールです。ディレクトリ探索とファイル内容の要約を効率的に行うことができます。

## 🌟 主な機能

- 💡 インタラクティブなディレクトリ探索
- 🤖 GPTによる高度なコンテンツ要約
- 🌍 多言語サポート
- 📁 単一ファイルおよび一括処理モード
- ⚡ 効率的なキャッシュシステム
- 🔄 カスタマイズ可能なAIクエリ

## 🚀 インストール

```bash
# リポジトリのクローン
git clone https://github.com/todorokitoyotaka/nexplorer
cd nexplorer

# 依存関係のインストール
cargo build --release

# バイナリを~/binディレクトリにインストール
mkdir -p ~/bin
cp target/release/nexplorer ~/bin/

# PATHが設定されていない場合は、~/.bashrcまたは~/.zshrcに以下を追加
echo 'export PATH="$HOME/bin:$PATH"' >> ~/.bashrc  # または ~/.zshrc

# 変更を反映
source ~/.bashrc  # または source ~/.zshrc

# 実行権限の設定
chmod +x ~/bin/nexplorer
```

注: インストール後、新しいターミナルを開くか、sourceコマンドでPATH設定を反映させてください。

## 🔧 環境設定

1. OpenAI APIキーの設定:
```bash
export OPENAI_API_KEY="your-api-key"
```

## 📖 使い方

### 基本的な使用方法

```bash
# 現在のディレクトリの探索と要約
cargo run -- --ai .

# 特定のディレクトリの探索
cargo run -- --ai /path/to/directory

# 特定の言語での要約生成
cargo run -- --ai --language japanese /path/to/directory
```

### 詳細オプション

```bash
# 要約の長さを指定
cargo run -- --ai --summary-length short /path/to/directory

# キャッシュの更新を強制
cargo run -- --ai --update /path/to/directory

# カスタムAIクエリの実行
cargo run -- --ai-whole "セキュリティの観点から分析してください" /path/to/directory
```

## 🎯 主要機能の詳細

### サマリー生成
- ファイル内容の自動要約
- 3段階の要約長（short/medium/long）
- コードファイルに最適化された要約アルゴリズム

### キャッシュシステム
- `.cache`ディレクトリでの効率的な要約保存
- コンテンツハッシュによる変更検知
- 言語設定と要約長の追跡

### 多言語サポート
- 複数言語での要約生成
- インタラクティブな言語切り替え
- カスタム言語クエリのサポート

## ⚙️ 設定オプション

| オプション | 説明 | デフォルト値 |
|------------|------|--------------|
| `--ai` | GPT要約の有効化 | 無効 |
| `--language` | 出力言語の指定 | 英語 |
| `--summary-length` | 要約の長さ（short/medium/long） | medium |
| `--update` | キャッシュの強制更新 | 無効 |
| `--ai-whole` | カスタムAIクエリの実行 | なし |

## 🔍 キャッシュシステム

キャッシュは`.cache`ディレクトリに保存され、以下の情報を追跡します：
- ファイルのコンテンツハッシュ
- 選択された言語
- 要約の長さ
- 生成された要約

## 📝 使用例

### プロジェクト全体の分析
```bash
cargo run -- --ai-whole "プロジェクト構造を分析してください" .
```

### 特定言語でのファイル要約
```bash
cargo run -- --ai --language japanese src/main.rs
```

### キャッシュの更新
```bash
cargo run -- --ai --update --language japanese .
```

## 🤝 貢献について

プロジェクトへの貢献を歓迎します：
1. このリポジトリをフォーク
2. 新しいブランチを作成 (`git checkout -b feature/amazing-feature`)
3. 変更をコミット (`git commit -m 'Add amazing feature'`)
4. ブランチにプッシュ (`git push origin feature/amazing-feature`)
5. プルリクエストを作成

## 📜 ライセンス

このプロジェクトはMITライセンスの下で公開されています。

## 🙏 謝辞

- OpenAI - GPTモデルの提供
- Rustコミュニティ - 素晴らしいツールとライブラリの提供

<a name="english"></a>
# English

An intelligent file explorer tool powered by GPT that enables efficient directory exploration and file content summarization.

## 🌟 Key Features

- 💡 Interactive Directory Exploration
- 🤖 Advanced GPT-powered Content Summarization
- 🌍 Multilingual Support
- 📁 Single File and Batch Processing Modes
- ⚡ Efficient Cache System
- 🔄 Customizable AI Queries

## 🚀 Installation

```bash
# Clone the repository
git clone https://github.com/todorokitoyotaka/nexplorer
cd nexplorer

# Install dependencies
cargo build --release

# Install binary to ~/bin directory
mkdir -p ~/bin
cp target/release/nexplorer ~/bin/

# Add to PATH in ~/.bashrc or ~/.zshrc if not already set
echo 'export PATH="$HOME/bin:$PATH"' >> ~/.bashrc  # or ~/.zshrc

# Apply changes
source ~/.bashrc  # or source ~/.zshrc

# Set execution permissions
chmod +x ~/bin/nexplorer
. ~/.bashrc
```

Note: After installation, open a new terminal or use the source command to reflect PATH settings.

## 🔧 Configuration

1. Set up OpenAI API key:
```bash
export OPENAI_API_KEY="your-api-key"
```

## 📖 Usage

### Basic Usage

```bash
# Explore and summarize current directory
cargo run -- --ai .

# Explore specific directory
cargo run -- --ai /path/to/directory

# Generate summaries in specific language
cargo run -- --ai --language english /path/to/directory
```

### Advanced Options

```bash
# Specify summary length
cargo run -- --ai --summary-length short /path/to/directory

# Force cache update
cargo run -- --ai --update /path/to/directory

# Execute custom AI query
cargo run -- --ai-whole "Analyze from a security perspective" /path/to/directory
```

## 🎯 Detailed Features

### Summary Generation
- Automatic file content summarization
- Three summary lengths (short/medium/long)
- Optimized summarization algorithm for code files

### Cache System
- Efficient summary storage in `.cache` directory
- Content change detection via hashing
- Language preference and summary length tracking

### Multilingual Support
- Summary generation in multiple languages
- Interactive language switching
- Custom language query support

## ⚙️ Configuration Options

| Option | Description | Default Value |
|--------|-------------|---------------|
| `--ai` | Enable GPT summarization | Disabled |
| `--language` | Specify output language | English |
| `--summary-length` | Summary length (short/medium/long) | medium |
| `--update` | Force cache update | Disabled |
| `--ai-whole` | Execute custom AI query | None |

## 🔍 Cache System

The cache is stored in the `.cache` directory and tracks:
- File content hashes
- Selected language
- Summary length
- Generated summaries

## 📝 Usage Examples

### Project-wide Analysis
```bash
cargo run -- --ai-whole "Analyze project structure" .
```

### File Summary in Specific Language
```bash
cargo run -- --ai --language english src/main.rs
```

### Cache Update
```bash
cargo run -- --ai --update --language english .
```

## 🤝 Contributing

Contributions are welcome:
1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit changes (`git commit -m 'Add amazing feature'`)
4. Push to branch (`git push origin feature/amazing-feature`)
5. Create Pull Request

## 📜 License

This project is licensed under the MIT License.

## 🙏 Acknowledgments

- OpenAI - For providing GPT models
- Rust Community - For excellent tools and libraries
