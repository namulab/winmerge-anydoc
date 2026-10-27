# anydoc の自動更新・リリース

`.github/workflows/release.yml` の **Build and Release** が毎日 09:23 JST
（00:23 UTC）に crates.io の anydoc を確認します。GitHub Actions の混雑により
実行が遅れることがあります。GitHub 上のタグではなく、実際に Cargo で取得できる
正式版の公開を更新の基準とします。プレリリースと yanked（撤回済み）版は対象外です。

## 更新時の処理

1. crates.io の最新正式版と `Cargo.lock` の anydoc を数値で比較します。
2. 新しい版がある場合、`Cargo.toml` の anydoc をその版に固定し、
   `cargo update --package anydoc --precise VERSION` でロックファイルを更新します。
3. プロジェクトのパッチバージョンを 1 上げます（例: `0.1.1 → 0.1.2`）。
   複数の anydoc リリースがあった場合も最新の 1 版を取り込み、1 回だけ上げます。
4. `cargo build --release --locked` と `cargo test --locked` を実行します。
   テストには実際の DLL を読み込む COM スモークテストが含まれます。
5. cargo-about で `THIRD_PARTY_NOTICES.md` を再生成し、配布 ZIP を作成します。
6. 更新ファイルを main にコミットし、バージョンタグとともに atomic push します。
7. GitHub Release をドラフトで作成し、ZIP のアップロード後に公開します。

更新がない場合はコミットもリリースも作成しません。新しい anydoc が API の変更や
Rust バージョン要件の変更を含み、ビルド・テストに失敗した場合も公開しません。
その場合は Actions のログを確認し、必要なコードやツールチェーンの変更を手動で行います。
anydoc 自体のメジャー・マイナー更新も、上記の検証に成功すれば取り込み対象になります。

## 更新履歴の形式

自動リリースの本文には、更新前後の anydoc のバージョンと上流へのリンクを記載します。
GitHub のコミット一覧の自動生成は使用しません。

> ドキュメント変換ライブラリ anydoc を 0.1.9 から 0.2.4 に更新しました。
>
> 詳細は [anydoc の更新履歴](https://github.com/firecrawl/anydoc/releases)をご覧ください。

バージョンはリリースタグとその親コミットの `Cargo.lock` から取得するため、公開の
再実行時にも同じ内容になります。ドラフトから再開する場合もこの形式に更新します。
手動リリースなどで anydoc の変更がない場合は「anydoc VERSION を使用しています。」
と記載します。上流の修正内容や効果を推測して記載することはありません。

## 有効化と手動実行

この変更を GitHub の `main`（デフォルトブランチ）へ反映すると定期実行が有効になります。
Actions が有効で、リポジトリのポリシーがワークフローの `contents: write` を許可し、
GitHub Actions のトークンで main と `v*` タグに push できる必要があります。
保護ルールが直接 push を禁止している場合は、管理者が運用方針を決めて設定してください。
追加の PAT や外部サーバーは不要です。fork では公開ジョブは実行しません。

即時実行するには **Actions → Build and Release → Run workflow → main** を選択します。
これは更新がある場合に実際に公開する操作です。確認だけなら、ローカルで次を実行できます
（Python 3.11 以上。ファイルの変更・push・公開は行いません）。

```console
python scripts/auto_release.py check
```

公開が途中で失敗した場合、同じワークフローを再実行できます。main の現在のバージョンに
対応するタグがあり、Release が未作成またはドラフトなら、そのタグのソースをチェックアウト
して公開を再開します。公開済みの Release は上書きしません。検証中に main が更新された場合は
atomic push が失敗し、他の変更を上書きしません。次回実行では最新の main からやり直します。
従来の `v*` タグの手動 push によるリリースも引き続き利用できます。

標準の `GITHUB_TOKEN` による push は別の push ワークフローを起動しないため、
自動更新と公開は同じワークフロー内で完結させています。
([GitHub の仕様](https://docs.github.com/en/actions/concepts/security/github_token))

公開リポジトリでは、リポジトリの活動が 60 日間ないと GitHub が定期ワークフローを
自動で無効化します。長期間更新がない場合は Actions で有効状態を確認し、必要に応じて
再有効化してください。
([定期実行の仕様](https://docs.github.com/en/actions/reference/workflows-and-actions/events-that-trigger-workflows#schedule))

## スクリプトのテスト

```console
python -m unittest discover -s scripts -p "test_*.py" -v
```

ネットワークや GitHub への書き込みを行わず、正式版の選択、パッチ更新、失敗時の復元、
二重公開の回避、公開の再開を検証します。
