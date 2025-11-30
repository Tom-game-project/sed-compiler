use sed_compiler::*;

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

    // 解析された値を使う
    if args.verbose {
        println!("soil file {} -> sed file {}", args.input, args.output);
    }
}
