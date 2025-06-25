# sbv2-api で使用可能な辞書を作成する
## dependencies
```
brew install xz
```
## conda env
```
conda create -n sbv2-dict python=3.12
conda activate sbv2-dict
pip install chardet pandas
```
## mecab-ipadic / mecab-ipadic-neologd
```
git clone https://github.com/lindera/mecab-ipadic.git
git clone --depth 1 https://github.com/neologd/mecab-ipadic-neologd.git
```
## 統合辞書用ディレクトリの作成とデータ準備
- 以下のように実行すると、
- `mecab-ipadic` と `mecab-ipadic-neologd` を統合した辞書CSVファイル群が、
- `integrated_dict/dict_csv` に作成される。
```
mkdir integrated_dict
cd integrated_dict
mkdir source_csv
mkdir converted_csv
mkdir dict_csv
cp ../mecab-ipadic/*.csv source_csv/
cp ../mecab-ipadic/*.def source_csv/
cd ../mecab-ipadic-neologd/seed/
find . -name "*.csv.xz" -exec unxz {} \;
cp *.csv ../../integrated_dict/source_csv/
cd ../../
rm -rf mecab-ipadic
rm -rf mecab-ipadic-neologd
cd integrated_dict
cat <<EOF > convert_encoding.py
import os
import chardet

def convert_csv_to_utf8(source_dir, target_dir):
    for filename in os.listdir(source_dir):
        if not filename.endswith('.csv'):
            continue

        source_path = os.path.join(source_dir, filename)
        target_path = os.path.join(target_dir, filename)

        # 文字コードを自動判別
        with open(source_path, 'rb') as f:
            raw_data = f.read()
            encoding_info = chardet.detect(raw_data)
            if encoding_info is None:
                continue
            encoding = encoding_info['encoding']

        # UTF-8で再保存
        try:
            with open(source_path, 'r', encoding=encoding) as f_in:
                with open(target_path, 'w', encoding='utf-8', newline='') as f_out:
                    f_out.write(f_in.read())
            print(f"Converted {filename}: {encoding} -> UTF-8")
        except Exception as e:
            print(f"Error converting {filename}: {e}")

if __name__ == "__main__":
    convert_csv_to_utf8("source_csv", "converted_csv")
EOF
python convert_encoding.py
cat <<EOF > merge_csv.py
import pandas as pd
import os
import glob

def merge_csv_files_by_pos(input_dir, output_dir):
    # すべてのCSVファイルを読み込み、品詞ごとにグループ化
    all_dfs = []
    csv_files = list(glob.glob(os.path.join(input_dir, "*.csv")))
    total_files = len(csv_files)
    print(f"Found {total_files} CSV files to process...")

    for i, csv_file in enumerate(csv_files, 1):
        df = pd.read_csv(csv_file, encoding='utf-8', header=None)
        all_dfs.append(df)
        print(f"Processed {i}/{total_files} files...")

    if not all_dfs:
        print("No data found.")
        return

    combined = pd.concat(all_dfs, ignore_index=True)
    # 品詞カラム（例：4列目が品詞）
    pos_col = 4
    grouped = combined.groupby(combined[pos_col])
    total_entries = len(combined)
    print(f"\nGrouping by part of speech and saving results...")
    print(f"Total entries to classify: {total_entries}")

    processed = 0
    for pos, group in grouped:
        output_file = os.path.join(output_dir, f"{pos}.csv")
        group.to_csv(output_file, encoding='utf-8', header=False, index=False)
        processed_this_group = len(group)
        processed += processed_this_group
        progress = (processed / total_entries) * 100
        print(f"Saved {pos}.csv with {processed_this_group} entries. Progress: {progress:.1f}%")

    print(f"\nFinished! Total entries classified: {total_entries}")

if __name__ == "__main__":
    merge_csv_files_by_pos("converted_csv", "dict_csv")
EOF
python merge_csv.py
cp converted_csv/*.def dict_csv/
rm -rf converted_csv
rm -rf source_csv
```
- `integrated_dict/dict_csv` の中身は以下のような感じ。
```
.
├── char.def
├── feature.def
├── left-id.def
├── matrix.def
├── pos-id.def
├── rewrite.def
├── right-id.def
├── unk.def
├── 副詞.csv
├── 助詞.csv
├── 動詞.csv
├── 名詞.csv
├── 記号.csv
├── その他.csv
├── 助動詞.csv
├── 形容詞.csv
├── 感動詞.csv
├── 接続詞.csv
├── 接頭詞.csv
├── 連体詞.csv
└── フィラー.csv
```
## オリジナル辞書の作成
- `integrated_dict/dict_csv` 内の辞書は `参考` として利用することにする。
- オリジナル辞書の保管場所作成（`.def` ファイルを確保）
```
cd ./sbv2-dict
mkdir -m 755 -p dicts
mkdir -m 755 -p output
cp ./integrated_dict/dict_csv/*.def ./dicts/
```
- ?????????
- ?????????
- ?????????
### jpreprocess を入手
```
cd ./sbv2-dict
wget https://github.com/jpreprocess/jpreprocess/releases/download/v0.12.0/jpreprocess-aarch64-apple-darwin.tgz -O jpreprocess.tgz
tar vxzf jpreprocess.tgz
rm -rf jpreprocess.tgz
xattr -d com.apple.quarantine ./jpreprocess/dict_tools
```
### 専用辞書CSVファイル作成
- まず、空ファイル作成
```
cd ./sbv2-dict
touch dicts/covy_dict.csv
```
- `./sbv2-dict/dicts/covy_dict.csv` に必要な単語を登録してく
- 例えば「お電話」を登録する場合
    1. まず `./sbv2-dict/integrated_dict/dict_csv/名詞.csv` の中で「電話」を探す
    2. 「電話」が以下のように見つかる
        ```
        電話,1285,1285,9011,名詞,一般,*,*,*,*,電話,デンワ,デンワ
        ```
    3. 以下のように、「お電話」への編集と「<アクセント位置>/<モーラ数>,<アクセント結合規則>,<チェーンフラグ>」の追加
        ```
        お電話,1285,1285,10000,名詞,一般,*,*,*,*,お電話,オデンワ,オデンワ,2/4,*,*
        ```
    4. アクセント結合規則とチェーンフラグは基本的に「*」でいいと思う。
### バイナリ辞書へ変換
- 以下のコマンドで変換できる
```
cd ./sbv2-dict
make build
```
- この段階で、`./output` 内は以下の状態
```
.
├── all.bin
├── char_def.bin
├── dict.da
├── dict.vals
├── dict.words
├── dict.wordsidx
├── matrix.mtx
└── unk.bin
```
- 以下のように実行できる
```
./jpreprocess/jpreprocess -d ./output "お電話ください"

お電話,名詞,一般,*,*,*,*,お電話,オデンワ,オデンワ,2/4,*,-1
ください,名詞,*,*,*,*,*,ください,*,,0/0,*,-1
[NJD]
お電話,名詞,一般,*,*,*,*,お電話,オデンワ,オデンワ,2/4,*,-1
ください,フィラー,*,*,*,*,*,ください,クダサイ,クダサイ,0/4,*,1
```
- `sbv2-api` で使用するのは全てをシリアライズして単一バイナリ化した `all.bin`
