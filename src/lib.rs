pub mod code_gen;
pub mod val_resolver;

pub use code_gen::{
    build_ast_test02, 
    build_ast_test03, 
    build_ast_test04};
pub use val_resolver::*;


pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;


    #[test]
    fn it_works00() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn it_works01() {
        let mut file = File::create("./labo6.sed")
            .expect("ファイルが開けませんでした");  
        let a = build_ast_test02();
        file.write_all(a.as_bytes())
            .expect("書き込みに失敗しました");
    }

    #[test]
    fn it_works02() {
        let mut file = File::create("./labo6.sed")
            .expect("ファイルが開けませんでした");  
        let a = build_ast_test03();
        file.write_all(a.as_bytes())
            .expect("書き込みに失敗しました");
    }

    #[test]
    fn it_works03() {
        let mut file = File::create("./labo6.sed")
            .expect("ファイルが開けませんでした");  
        let a = build_ast_test04();
        file.write_all(a.as_bytes())
            .expect("書き込みに失敗しました");
    }
}
