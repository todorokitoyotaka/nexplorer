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

## ⚙️ 設定オプション

```bash
nexplorer [オプション] <パス>

オプション:
  --ai                    GPTによるファイル要約機能を有効化
  --ai-query <クエリ>      カスタム要約クエリを指定
                         例: --ai-query "このコードの主な機能は何ですか？"
  
  --ai-whole <クエリ>      ディレクトリ内の全ファイルを一括で分析
                         例: --ai-whole "セキュリティの観点から分析してください"
  
  --max-depth <数値>      探索する最大ディレクトリ深度（デフォルト: 3）
                         例: --max-depth 5
  
  --summary-length <長さ>  要約の長さを指定
                         選択肢:
                         - short  : 約50語
                         - medium : 約100語（デフォルト）
                         - long   : 約200語
  
  --language <言語>       要約の言語を指定（デフォルト: english）
                         例: --language japanese
  
  --update               キャッシュを強制的に更新
```

### 使用例

```bash
# 基本的な使用方法（現在のディレクトリを探索）
nexplorer .

# GPT要約機能を有効化して特定のディレクトリを探索
nexplorer --ai /path/to/project

# 日本語で長めの要約を生成
nexplorer --ai --language japanese --summary-length long .

# カスタムクエリで特定の分析を実行
nexplorer --ai-query "このコードのパフォーマンスに関する問題点は？" src/

# プロジェクト全体のセキュリティ分析
nexplorer --ai-whole "セキュリティの脆弱性について分析してください" .
```

## 🚀 インストール

```bash
# リポジトリのクローン
git clone https://github.com/todorokitoyotaka/nexplorer.git
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

## ❗ トラブルシューティング

### APIキー関連の問題

1. **エラー**: `OPENAI_API_KEY environment variable is not set`
   - **解決方法**: 
     - 環境変数に`OPENAI_API_KEY`を設定してください
     - `export OPENAI_API_KEY='your-api-key'`を実行するか、~/.bashrcに追加してください
     - シェルを再起動して変更を反映させてください

2. **エラー**: `Invalid API key provided`
   - **解決方法**:
     - APIキーが正しく設定されているか確認してください
     - OpenAIのダッシュボードで有効なAPIキーであることを確認してください

### キャッシュ関連の問題

1. **問題**: キャッシュが更新されない
   - **解決方法**:
     - `--update`フラグを使用してキャッシュを強制的に更新: 
       ```bash
       nexplorer --ai --update <パス>
       ```
     - `.cache`ディレクトリを削除して再実行してください

2. **問題**: キャッシュディレクトリのパーミッションエラー
   - **解決方法**:
     - キャッシュディレクトリの権限を確認:
       ```bash
       chmod -R 755 .cache
       ```

### パフォーマンスと制限

1. **問題**: 大きなファイルの処理が遅い
   - **解決方法**:
     - ファイルサイズの制限（1MB）を超えていないか確認
     - `--max-depth`オプションで探索深度を制限
     - バッチ処理モード（`--ai-whole`）の使用を検討

2. **問題**: メモリ使用量が多い
   - **解決方法**:
     - 一度に処理するファイル数を制限
     - 定期的にキャッシュをクリア

### 言語とフォーマット

1. **問題**: 要約が指定した言語で出力されない
   - **解決方法**:
     - `--language`オプションが正しく設定されているか確認
     - キャッシュを更新して再試行
     - サポートされている言語であることを確認

2. **問題**: 文字化けが発生する
   - **解決方法**:
     - ターミナルのエンコーディングをUTF-8に設定
     - ロケール設定を確認

### その他の一般的な問題

1. **問題**: 依存関係のインストールエラー
   - **解決方法**:
     - Rustとcargoが最新版かを確認
     - `cargo clean && cargo build --release`を実行

2. **問題**: PATHの設定が反映されない
   - **解決方法**:
     - シェルの設定ファイル（.bashrcまたは.zshrc）を確認
     - `source ~/.bashrc`（または`~/.zshrc`）を実行

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

## ⚙️ Configuration Options

```bash
nexplorer [OPTIONS] <PATH>

Options:
  --ai                    Enable GPT-powered file summarization
  --ai-query <QUERY>      Specify custom summarization query
                         Example: --ai-query "What are the main functions of this code?"
  
  --ai-whole <QUERY>      Analyze all files in directory as a batch
                         Example: --ai-whole "Analyze from a security perspective"
  
  --max-depth <NUMBER>    Maximum directory depth to explore (default: 3)
                         Example: --max-depth 5
  
  --summary-length <LENGTH> Specify summary length
                         Choices:
                         - short  : ~50 words
                         - medium : ~100 words (default)
                         - long   : ~200 words
  
  --language <LANG>       Specify summary language (default: english)
                         Example: --language spanish
  
  --update               Force update cache entries
```

### Usage Examples

```bash
# Basic usage (explore current directory)
nexplorer .

# Explore specific directory with GPT summarization
nexplorer --ai /path/to/project

# Generate longer summaries in Spanish
nexplorer --ai --language spanish --summary-length long .

# Run custom analysis query
nexplorer --ai-query "What are the performance concerns in this code?" src/

# Analyze entire project for security
nexplorer --ai-whole "Analyze for security vulnerabilities" .
```

## 🚀 Installation

```bash
# Clone the repository
git clone https://github.com/todorokitoyotaka/nexplorer.git
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
```

## ❗ Troubleshooting

### API Key Issues

1. **Error**: `OPENAI_API_KEY environment variable is not set`
   - **Solution**: 
     - Set the `OPENAI_API_KEY` environment variable
     - Run `export OPENAI_API_KEY='your-api-key'` or add it to ~/.bashrc
     - Restart your shell to apply changes

2. **Error**: `Invalid API key provided`
   - **Solution**:
     - Verify that your API key is set correctly
     - Check if the API key is valid in your OpenAI dashboard

### Cache-Related Issues

1. **Issue**: Cache not updating
   - **Solution**:
     - Use the `--update` flag to force cache refresh:
       ```bash
       nexplorer --ai --update <path>
       ```
     - Delete the `.cache` directory and run again

2. **Issue**: Cache directory permission errors
   - **Solution**:
     - Check cache directory permissions:
       ```bash
       chmod -R 755 .cache
       ```

### Performance and Limitations

1. **Issue**: Slow processing of large files
   - **Solution**:
     - Check if file size exceeds limit (1MB)
     - Limit exploration depth with `--max-depth`
     - Consider using batch mode (`--ai-whole`)

2. **Issue**: High memory usage
   - **Solution**:
     - Limit the number of files processed at once
     - Clear cache periodically

### Language and Formatting

1. **Issue**: Summaries not in specified language
   - **Solution**:
     - Verify `--language` option is set correctly
     - Update cache and retry
     - Ensure language is supported

2. **Issue**: Character encoding problems
   - **Solution**:
     - Set terminal encoding to UTF-8
     - Check locale settings

### General Issues

1. **Issue**: Dependency installation errors
   - **Solution**:
     - Verify Rust and cargo are up to date
     - Run `cargo clean && cargo build --release`

2. **Issue**: PATH settings not taking effect
   - **Solution**:
     - Check shell configuration file (.bashrc or .zshrc)
     - Run `source ~/.bashrc` (or `~/.zshrc`)

## 📜 License

This project is licensed under the MIT License.

## 🙏 Acknowledgments

- OpenAI - For providing GPT models
- Rust Community - For excellent tools and libraries
