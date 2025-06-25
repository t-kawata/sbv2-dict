use lindera_dictionary::dictionary::prefix_dictionary::PrefixDictionary;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::Command;
use yada::DoubleArray;

#[derive(Clone, Serialize, Deserialize)]
pub struct UserDictionary {
    pub dict: PrefixDictionary,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. dict_tools build lindera を実行
    let status = Command::new("./jpreprocess/dict_tools")
        .args(&["build", "lindera", "./dicts", "./output"])
        .status()?;

    if !status.success() {
        return Err("dict_tools build failed".into());
    }

    let outpath = "./output/all.bin";

    // 2. 複数ファイルからall.binを作成
    create_all_bin_from_multiple_files(Path::new("./output"), Path::new(outpath))?;

    println!("Dictionary build suceeded -> ./output/all.bin");
    Ok(())
}

fn create_all_bin_from_multiple_files(
    input_dir: &Path,
    output_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    // 1. 各バイナリファイルを読み込み
    let dict_da_data = fs::read(input_dir.join("dict.da"))?;
    let dict_vals_data = fs::read(input_dir.join("dict.vals"))?;
    let dict_words_data = fs::read(input_dir.join("dict.words"))?;
    let dict_wordsidx_data = fs::read(input_dir.join("dict.wordsidx"))?;

    // 2. PrefixDictionaryを構築
    let prefix_dict = PrefixDictionary {
        da: DoubleArray::new(dict_da_data.into()),
        vals_data: dict_vals_data.into(),
        words_idx_data: dict_wordsidx_data.into(),
        words_data: dict_words_data.into(),
        is_system: false, // ユーザー辞書なのでfalse
    };

    // 3. UserDictionaryを作成
    let user_dict = UserDictionary { dict: prefix_dict };

    // 4. bincodeでシリアライズ
    let serialized_data = bincode::serialize(&user_dict)?;

    // 5. all.binとして保存
    fs::write(output_path, serialized_data)?;

    Ok(())
}
