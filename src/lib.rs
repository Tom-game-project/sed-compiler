pub mod ast;
pub use ast::build_ast_test;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works00() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn it_works01() {
        build_ast_test();
    }
}
