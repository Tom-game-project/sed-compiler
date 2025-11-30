use sed_compiler::{
    compiler::{compiler_frontend},
};

use clap::Parser;

/// soilcはsedコンパイラを制御するためのUIです。
/// soilプログラムをsedにトランスパイルすることができます
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)] // バージョン情報や説明文を自動設定
struct Args {
    /// コンパイルしたいsoilファイル
    #[arg(short, long)]
    input: String,

    /// 出力されるsedファイルの名前
    #[arg(short, long, default_value_t = String::from("out.sed"))]
    output: String,

    /// 詳細表示モードかどうか (フラグ)
    /// 値を取らず、存在するだけで true になる bool 型
    #[arg(short, long)]
    verbose: bool,
}

fn main() {
    // 引数を解析して構造体に変換
    let args = Args::parse();

    let code = 
        std::fs::read_to_string(args.input)
        .expect("Failed to Open File"); // TODO: ファイルオープンの失敗を処理

    // ソースに基づいてIRを生成する
    // CompileBuilderの中に中間表現IRの情報を含む
    let r_ir = compiler_frontend(&code);

    match  r_ir {
        Ok(compiler_builder) => {
            // リターンアドレス、ラベルの解決をする
            let assembled = compiler_builder.assemble();
            // 解決済みのIRを表示する
            if args.verbose {
                assembled.resolved_show_table();
            }
            // 解決の終わったIRからsedスクリプトを生成する
            let generated = assembled.generate();
            match generated {
                Ok(generated_sed_code) => {
                    std::fs::write(args.output, generated_sed_code)
                        .expect("Failed to write file");
                }
                Err(err) => {
                    println!("{:?}", err);
                }
            }
        }
        Err(err) => {
            println!("{:?}", err);
        }
    }
}
