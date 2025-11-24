// compileが成功するものは、それらの成果物が予想通りの挙動を示すかテストする
// main.rs や lib.rs と同じ階層、またはテスト用のモジュール内に記述します
mod gen_test;

#[cfg(test)]
mod command_test {
    use std::fmt::format;
    use std::process::{Command, Stdio};
    use std::io::{Read, Write};
    use sed_compiler::compiler::compiler_frontend;

    struct DebugCompilerError {
        error_msg: String
    }

    fn sed_operation_test(code: &str, input:&str) -> Result<String, DebugCompilerError>{
        match compiler_frontend(code) {
            Ok(compiler_builder) => {
                let assembled = compiler_builder.assemble();
                // assembled.resolved_show_table();
                println!("success to compile soil program!");
                let generated = assembled.generate();
                match generated {
                    Ok(generated_sed_code) => {
                        let mut child = Command::new("sed")
                            // .arg("-f")
                            // .arg("sed/basic_operations.sed")
                            .arg(generated_sed_code)
                            .stdin(Stdio::piped())
                            .stdout(Stdio::piped())
                            .spawn().expect("子プロセスの生成に失敗しました");

                        {
                            let mut stdin = child.stdin.take().unwrap();
                            // let data = format!("~{:032b}~{:032b}", 123456789, 987654321);   // echo hello の代わり
                            stdin.write_all(input.as_bytes()).expect("stdinの書き込みに失敗しました");
                        }

                        let mut output = String::new();
                        if let Some (a) = child.stdout.as_mut() {
                            a.read_to_string(&mut output).expect("標準出力に失敗しました");
                        } else {
                        }

                        // 4. コマンド終了を待つ
                        let status = child.wait().expect("コマンドの終了前にエラーが発生しました");
                        println!("status = {:?}", status);

                        // 5. Rust 側で出力を利用可能
                        println!("output = {:?}", output);
                        // assert_eq!(output, expected_output);
                        // assert_eq!(status., expected_status_code);
                        return Ok(output);
                    }
                    Err(err) => {
                        return Err(DebugCompilerError { error_msg: format!("{:?}", err)})
                    }
                }
            }
            Err(err) => {
                println!("{:?}", err);
                return Err(DebugCompilerError { error_msg: format!("{:?}", err)});
            }
        }
    }

    /// ```sh
    /// cargo test command_test
    /// ```
    #[test]
    fn test_sed_script_with_timeout00() {
        // --- 設定 ---
        // 期待する標準出力
        let expected_output = "~00000000000000000000000000000000~00000000000000000000000000001001~00000000000000000000000000001001;";
        // 期待する終了ステータス (0は正常終了)
        let expected_status_code = 0;

        // --- 実行 ---
        // 実行したいコマンドを設定
        let output = Command::new("timeout")
            .arg("3s")
            .arg("sed")
            .arg("-f")
            .arg("sed/c_example.sed")
            .arg("in")
            .output() // コマンドを実行し、出力全体を待つ
            .expect("コマンドの実行に失敗しました"); // 実行失敗時にパニック

        // --- 検証 ---

        // 1. 終了ステータスの検証
        // output.status.code() は Option<i32> を返す
        let actual_status_code = output.status.code().unwrap_or(-1);

        // 期待した終了コードと実際の終了コードを比較
        assert_eq!(
            actual_status_code, expected_status_code,
            "終了ステータスが期待と異なります。タイムアウト(124)した可能性があります。"
        );

        // 2. 標準出力の検証
        // output.stdout は Vec<u8> (バイト列) なので、UTF-8文字列に変換
        let actual_output =
            String::from_utf8(output.stdout).expect("標準出力をUTF-8文字列に変換できませんでした");

        // 改行などを除去して比較する
        assert_eq!(
            actual_output.trim(),
            expected_output,
            "標準出力が期待と異なります。"
        );
    }

    #[test]
    fn test_sed_script_with_timeout01() {
        let expected_output = format!("~{:032b}~{:032b}~{:032b};", 0, 9, 9);
        let expected_status_code = 0;

        let mut child = Command::new("sed")
            .arg("sed/basic_operations.sed")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn().expect("子プロセスの生成に失敗しました");

        {
            let mut stdin = child.stdin.take().unwrap();
            let data = format!("~{:032b}~{:032b}", 123456789, 987654321);   // echo hello の代わり
            stdin.write_all(data.as_bytes()).expect("stdinの書き込みに失敗しました");
        }

        let mut output = String::new();
        if let Some (a) = child.stdout.as_mut() {
            a.read_to_string(&mut output).expect("標準出力に失敗しました");
        } else {
        }

        // 4. コマンド終了を待つ
        let status = child.wait().expect("コマンドの終了前にエラーが発生しました");
        println!("status = {:?}", status);

        // 5. Rust 側で出力を利用可能
        println!("output = {:?}", output);
        assert_eq!(output, expected_output);
        // assert_eq!(status., expected_status_code);
    }

    #[test]
    fn test_sed_script_with_timeout02(){ 
        use std::fs;
        let code = fs::read_to_string("soil/basic_operations.soil").expect("ファイルの読み込みに失敗しました");
        let input_args = &format!("~{:032b}~{:032b}", 123456789, 987654321);
        let expected_output = format!("~{:032b}~{:032b}~{:032b};", 0, 9, 9);
        match sed_operation_test(&code, input_args) {
            Ok(result) => {
                println!("result: {}", result);
                assert_eq!(expected_output, result);
            }
            Err(err) => {
                println!("{}", err.error_msg);
            }
        }
    }
}
