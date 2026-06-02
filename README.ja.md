# Structure Heatmap

Structure Heatmap は、コードベース内のファイルやディレクトリの大きさを可視化し、構造上の偏りを見つけるための軽量な分析ツールです。

依存関係や複雑度を評価するのではなく、次の問いに集中します。

> コードベースのどこに、周囲と比べて不自然に大きな領域があるか？

大きなファイルやディレクトリを自動的に悪いものとして扱うのではなく、設計レビューやリファクタリングで人間が確認すべき場所を見つけやすくします。

## 機能・ユースケース

Structure Heatmap は、次のような用途を想定しています。

* ディレクトリごとのファイル数と行数を確認する
* 周囲と比べて大きいファイルやディレクトリを検出する
* 空行やコメント行を含めるかどうかを指定して行数を集計する
* 設計レビュー前に、肥大化した領域を把握する
* AI コーディングアシスタントに、コードベースの構造的な偏りを伝える
* 継続的に計測し、アーキテクチャ上の重みの変化を追跡する
* レガシーシステムで、責務が集中していそうな場所を素早く見つける

検出対象は、たとえば次のような構造上の偏りです。

* 周囲のファイルより大きいファイル
* 同階層の中で大きく膨らんだディレクトリ
* ファイル数や行数が集中している領域
* 時間とともに責務が集まりやすい場所

一方で、次の分析は対象外です。

* 依存関係分析
* 循環依存の検出
* アーキテクチャルールの強制
* 静的コード解析
* リント
* 複雑度スコアリング
* 自動リファクタリング提案

これらは既存の専門ツールに任せ、Structure Heatmap は構造的なサイズ分布に集中します。

## インストール

GitHub Releases から利用している OS 向けのバイナリをダウンロードし、実行できる場所に置いてください。

利用している OS 向けのバイナリがない場合は、ソースコードからコンパイルしてください。

```sh
git clone <repository-url>
cd structure-heatmap
cargo build --release
```

## 使い方

分析したいコードベースのルートディレクトリを指定して実行します。

```sh
sheat ./src
```

出力形式を指定する場合は `--format` を使います。

```sh
sheat ./src --format markdown
sheat ./src --format json
sheat ./src --format html
```

行数の数え方を指定する場合は `--line-mode` を使います。

```sh
sheat ./src --line-mode physical
sheat ./src --line-mode sloc
```

`physical` は空行やコメント行を含めた行数です。`sloc` は空行を除外し、認識済み言語では行コメントとブロックコメントのみからなる行も除外した実質的なコード行数です。コードと同じ行にあるコメントは除外されません。

デフォルトは `physical` です。

Markdown レポートをファイルに保存する例:

```sh
sheat ./src --format markdown --output structure-report.md
```

見た目を重視した HTML レポートを保存する例:

```sh
sheat ./src --format html --output structure-report.html
```

出力例:

```markdown
## Directory Summary

| Path | Files | Lines |
| --- | ---: | ---: |
| src/components | 42 | 8,124 |
| src/domain | 18 | 1,327 |

## Hotspots

| Path | Kind | Lines | Median | Ratio |
| --- | --- | ---: | ---: | ---: |
| src/components/UserEditPage.tsx | file | 842 | 176 | 4.8x |
```

## 設定・オプション

設定は、CLI オプションまたは設定ファイルで指定できます。

主なオプション:

| オプション | 説明 |
| --- | --- |
| `--format <markdown\|json\|html>` | 出力形式を指定する |
| `--output <path>` | 結果を書き出すファイルを指定する |
| `--ignore <pattern>` | 分析対象から除外するパターンを指定する |
| `--include-hidden` | 隠しファイルや隠しディレクトリも分析対象に含める |
| `--line-mode <physical\|sloc>` | 行数の数え方を指定する。デフォルトは `physical` |
| `--min-ratio <number>` | ホットスポットとして扱う倍率のしきい値を指定する |
| `--max-depth <number>` | 集計するディレクトリ階層の深さを指定する |
| `--config <path>` | 設定ファイルのパスを指定する |

`.github`、`.vscode`、`.sheatrc` などの隠しファイルや隠しディレクトリは、デフォルトで分析対象から除外されます。Structure Heatmap は、ツールや環境設定ではなく、アプリケーションコードの構造を把握することを目的としているためです。隠しパスも構造分析に含めたい場合は、`--include-hidden` を指定します。

`.gitignore` に指定されているファイルやディレクトリは、自動的に分析対象から除外されます。

フィルタリングオプションの例:

```sh
sheat . --ignore node_modules --ignore dist
sheat . --include-hidden
```

JSON 出力は、自動分析や CI での利用に向いています。

```sh
sheat . --format json --output structure-report.json
```

HTML 出力は、人間が見やすいレポートや共有用の成果物に向いています。

```sh
sheat . --format html --output structure-report.html
```

### 設定ファイル

設定ファイルを使う場合は、`sheat` コマンドを実行するディレクトリ（カレントディレクトリ）に `.sheatrc` を置きます。

```ini
target = .
format = html
output = structure-report.html
ignore = node_modules
ignore = dist
include-hidden = false
line-mode = physical
min-ratio = 3
max-depth = 5
```

設定ファイルを明示する場合:

```sh
sheat --config .sheatrc
```

CLI オプションと設定ファイルの両方を指定した場合は、CLI オプションを優先します。
