mod db;
pub mod defaults;
pub mod errors;
mod page;
mod bucket;
mod meta;
mod freelist;
mod config;
mod options;
mod transaction;
mod inner;
mod bytes;
mod sys;
mod context;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
