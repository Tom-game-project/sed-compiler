// compileが成功するものは、それらの成果物が予想通りの挙動を示すかテストする
// main.rs や lib.rs と同じ階層、またはテスト用のモジュール内に記述します
mod gen_test;
use gen_test::*;
#[cfg(test)]
mod command_test {
    use std::process::Command;

    /// ```sh
    /// cargo test command_test
    /// ```
    #[test]
    fn test_sed_script_with_timeout() {
        // --- 設定 ---
        // 期待する標準出力
        let expected_output = "~hello~Tom~helloworldTom";
        // 期待する終了ステータス (0は正常終了)
        let expected_status_code = 0;

        // --- 実行 ---
        // 実行したいコマンドを設定
        let output = Command::new("timeout")
            .arg("1s")
            .arg("sed")
            .arg("-f")
            .arg("labo6.sed")
            .arg("in.txt")
            .output() // コマンドを実行し、出力全体を待つ
            .expect("コマンドの実行に失敗しました"); // 実行失敗時にパニック

        // --- 検証 ---
        
        // 1. 終了ステータスの検証
        // output.status.code() は Option<i32> を返す
        let actual_status_code = output.status.code().unwrap_or(-1);

        // 期待した終了コードと実際の終了コードを比較
        assert_eq!(
            actual_status_code,
            expected_status_code,
            "終了ステータスが期待と異なります。タイムアウト(124)した可能性があります。"
        );

        // 2. 標準出力の検証
        // output.stdout は Vec<u8> (バイト列) なので、UTF-8文字列に変換
        let actual_output = String::from_utf8(output.stdout)
            .expect("標準出力をUTF-8文字列に変換できませんでした");

        // 改行などを除去して比較する
        assert_eq!(
            actual_output.trim(),
            expected_output,
            "標準出力が期待と異なります。"
        );
    }

    #[test]
    fn test_sed_script_with_timeout01()
    {
         // --- 設定 ---
        // 期待する標準出力
        let expected_output = "~10101010110000100010~11101110111";
        // 期待する終了ステータス (0は正常終了)
        let expected_status_code = 0;

        // --- 実行 ---
        // 実行したいコマンドを設定
        let output = Command::new("timeout")
            .arg("1s")
            .arg("sed")
            .arg("-f")
            .arg("sed/mul.sed")
            .arg("in.txt")
            .output() // コマンドを実行し、出力全体を待つ
            .expect("コマンドの実行に失敗しました"); // 実行失敗時にパニック

        // --- 検証 ---
        
        // 1. 終了ステータスの検証
        // output.status.code() は Option<i32> を返す
        let actual_status_code = output.status.code().unwrap_or(-1);

        // 期待した終了コードと実際の終了コードを比較
        assert_eq!(
            actual_status_code,
            expected_status_code,
            "終了ステータスが期待と異なります。タイムアウト(124)した可能性があります。"
        );

        // 2. 標準出力の検証
        // output.stdout は Vec<u8> (バイト列) なので、UTF-8文字列に変換
        let actual_output = String::from_utf8(output.stdout)
            .expect("標準出力をUTF-8文字列に変換できませんでした");

        // 改行などを除去して比較する
        assert_eq!(
            actual_output.trim(),
            expected_output,
            "標準出力が期待と異なります。"
        );
       
    }
}
