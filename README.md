# ZDR055の映像データからGPXファイルを生成するやつ

```
$ ./zdr055_gpx --help
Usage: zdr055_gpx [OPTIONS] <PATH>

Arguments:
  <PATH>  

Options:
  -o, --output-path <OUTPUT_PATH>  [default: ./]
  -p, --parallel <PARALLEL>        [default: 8]
  -h, --help                       Print help



$ ./zdr055_gpx sd/NORMAL/Front/20250306_162217_F_M_Nor.AVI 
Processing file: sd/NORMAL/Front/20250306_162217_F_M_Nor.AVI
Extracted data from sd/NORMAL/Front/20250306_162217_F_M_Nor.AVI



$ ./zdr055_gpx sd/NORMAL/Front/
Processing directory: sd/NORMAL/Front/
Processing file: sd/NORMAL/Front/20250306_162319_F_E_Nor_s.AVI
Processing file: sd/NORMAL/Front/20250220_031901_F_M_Nor_Rst.AVI
Processing file: sd/NORMAL/Front/20250220_031659_F_M_Nor_Rst.AVI
Processing file: sd/NORMAL/Front/20250220_031931_F_M_Nor_Rst.AVI
Processing file: sd/NORMAL/Front/20250306_162421_F_M_Nor.AVI
Processing file: sd/NORMAL/Front/20250220_032103_F_M_Nor_Rst.AVI
Processing file: sd/NORMAL/Front/20250220_031729_F_M_Nor_Rst.AVI
Processing file: sd/NORMAL/Front/20250220_032002_F_M_Nor_Rst.AVI
Extracted data from sd/NORMAL/Front/20250220_031931_F_M_Nor_Rst.AVI
Extracted data from sd/NORMAL/Front/20250220_032002_F_M_Nor_Rst.AVI
Extracted data from sd/NORMAL/Front/20250220_031901_F_M_Nor_Rst.AVI
Extracted data from sd/NORMAL/Front/20250306_162421_F_M_Nor.AVI
...
Extracted data from sd/NORMAL/Front/20250220_031359_F_M_Nor_Rst.AVI
$
```

## 注意
* 実走行したデータで試してないので何が起きるかわかりません
    * 家で電源入れたときに採取したデータのみで試してる
* AVIインデックスの検出が超雑なので失敗するかもしれない
* 南半球とか日本国外で録画したデータで何が起きるかもわからない
    * このドラレコ自体が明らかに日本国内でのみ使用することを想定していると思うので勘弁してください