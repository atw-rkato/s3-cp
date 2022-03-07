# s3-cp

S3のバケット間で複数のオブジェクトを一括コピーする

## ビルド

Dockerまたはローカルでビルドします (どちらにせよ5分程度はかかると思います)。  
今のところは macOS (x86_64-apple-darwin) 版のみ対応してします。

### Dockerでビルド

```shell
./build-docker.sh
```

### ローカルでビルド

※ビルド用にcargoなどをインストールしますが、新規追加したものは最後に消しています。

```shell
./build-local.sh
```

ビルドが成功したら ./dist 配下にバイナリが出力されます。

```shell
ls ./dist
./dist/s3-cp -h  # ヘルプ
```

## 入力データの準備

以下の形式のCSVデータを用意します。  
※ 区切り文字は「`,`」  
※ フィールド間の空白は無視されます。

- 1列目 ... コピー元のバケット
- 2列目 ... コピー元のオブジェクトキー
- 3列目 ... コピー先のバケット
- 4列目 ... コピー先のオブジェクトキー

### データ例

通常 (コピー元のバケット, コピー元のオブジェクトキー, コピー先のバケット, コピー先のオブジェクトキー)

```text
sample-src-bucket, sample.txt, sample-dst-bucket, sample-copy.txt
```

オブジェクトが階層構造になっている場合は/で繋ぎます。

```text
sample-src-bucket2, Dir1/Dir2/sample2.txt, sample-dst-bucket2, Dir3/Dir4/sample2.txt
```

コピー先のオブジェクトキーが未指定の場合、コピー元のオブジェクトキーと同じキーになります (以下の例ならば Dir1/sample.txt)。

```text
sample-src-bucket, Dir1/sample.txt, sample-dst-bucket
```

## 実行

上記の入力データを標準入力に流し込んで実行します。   
入力データがファイルの場合は以下のようにリダイレクションしてください。

```shell
./dist/s3-cp --profile my-aws-profile --region ap-northeast-1 --verbose < sample.csv
```

コピーはデフォルトでは非同期に実行されます。  
同期的に実行したい場合は `--sync` オプションをつけてください。

```shell
./dist/s3-cp --sync < sample.csv
```

## 既知の不具合

現在、S3へのリクエストの同時実行数が多すぎると一部リクエストが正常終了しない場合があります。  
その場合は一旦下記のように`--max-pending` を指定し、同時実行数の上限を下げて再試行してみてください (初期値は128)。

```shell
./dist/s3-cp  --max-pending 64 < sample.csv
```
