# ZDR055の映像データからGPXファイルを生成するやつ
## 使い方
[workflow 実行結果](https://github.com/misodengaku/zdr055_gpx/actions/workflows/rust.yml)の Artifacts からいい感じのバイナリを取得するか、 Rust 1.87.0 以降とかの環境でいい感じにビルドしてください

```
$ ./zdr055_gpx --help
Usage: zdr055_gpx [OPTIONS] <PATH>

Arguments:
  <PATH>  

Options:
  -o, --output-path <OUTPUT_PATH>          [default: ./]
  -p, --parallel <PARALLEL>                [default: 8]
  -m, --merge                              
      --merge-threshold <MERGE_THRESHOLD>  [default: 6h]
  -d, --debug                              
  -h, --help                               Print help



$ ./zdr055_gpx -o output/ -m sd/NORMAL/Front/
Processing directory: sd/NORMAL/Front/
[1/726] Processing file: sd/NORMAL/Front/20250720_124810_F_S_Nor_Res.AVI
[2/726] Processing file: sd/NORMAL/Front/20250720_125902_F_S_Nor.AVI
[3/726] Processing file: sd/NORMAL/Front/20250720_125931_F_M_Nor.AVI
...
[724/726] Processing file: sd/NORMAL/Front/20250720_225035_F_M_Nor.AVI
[725/726] Processing file: sd/NORMAL/Front/20250720_225106_F_M_Nor.AVI
[726/726] Processing file: sd/NORMAL/Front/20250720_225136_F_E_Nor.AVI
--- Start merging logs ---
Output changed: output/20250720_124810_F_S_Nor_Res.gpx
Merging: sd/NORMAL/Front/20250720_124810_F_S_Nor_Res.AVI -> output/20250720_124810_F_S_Nor_Res.gpx
...
Merging: sd/NORMAL/Front/20250720_225136_F_E_Nor.AVI -> output/20250720_124810_F_S_Nor_Res.gpx
Saved GPX file: output/20250720_124810_F_S_Nor_Res.gpx
$
```

`./zdr055_gpx 2025mmdd_125902_F_S_Nor.AVI` のようにすることで COMTEC ZDR055 の出力した AVI ファイルから位置情報を抜き出し、 GPX ファイルとして書き出します。

処理対象をディレクトリにすると、指定したディレクトリ内に存在する AVI ファイルすべてに対して処理を行います。

処理対象がディレクトリのとき、 `-m` オプションを指定することでファイル群を連続するものとして処理を行い、連続すると思われる動画群の位置情報を1つの GPX ファイルへマージします。連続判定の基準は `--merge-threshold 30m` などとすることで変更できます。標準では欠測期間が6時間以内であれば連続するものとして扱います。

例えば、5日間の旅行に出かけたときの記録をまとめて処理する際、宿泊先で最低8時間の睡眠を取った場合などには `--merge-threshold 8h` などとすることでいい感じに分離できます。

## 注意
* そこまでしっかり試してるわけではないので上手くいかない可能性があります
* AVI インデックスの検出が超雑なので失敗するかもしれない
* 南半球とか日本国外で録画したデータで何が起きるかもわからない
    * このドラレコ自体が明らかに日本国内でのみ使用することを想定していると思うので勘弁してください
    * 映像に埋め込まれる時刻が JST 固定なので、おそらく考えるだけ無駄
* [ZDR-Viewer Type09](https://www.e-comtec.co.jp/0_recorder/viewer/ZDRviewerType09/viewer.html) の対応機種を見る限り以下の機種で動作するものと思われますが、 ZDR055 以外での録画データでは一切試していません。
  * ZDR058
  * ZDR055
  * ZDR048
  * ZDR045WL
  * ZDR045
  * ZDR043
  * ZDR038
  * ZDR027
  * ZDR018
  * ZDR017
  * AZDR48
  * DR-720KT-P
  * DR-620DS-P
