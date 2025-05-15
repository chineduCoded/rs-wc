mod argument_parser;
mod count_handling;
mod error_handling;
mod output_handling;


pub use argument_parser::parser;
pub use count_handling::counter;
pub use error_handling::error;
pub use output_handling::printer;


#[cfg(test)]
mod proptests {
    use proptest::prelude::*;
    use std::io::Cursor;
    use crate::parser::CountMode;
    use crate::count_handling::counter::{WcCounter, count_bytes, count_reader};

    proptest! {
        #[test]
        fn test_count_bytes_never_panics(bytes in any::<Vec<u8>>()) {
            let _ = count_bytes(&bytes, None, &[CountMode::Bytes]);
        }

        #[test]
        fn test_line_count_consistency(text in ".*") {
            let reader = Cursor::new(&text);
            let result = count_reader(reader, None, &[CountMode::Lines]).unwrap();
            
            let expected = text.matches('\n').count();
            if !text.is_empty() && !text.ends_with('\n') {
                assert_eq!(result.lines, expected + 1);
            } else {
                assert_eq!(result.lines, expected);
            }
        }

        #[test]
        fn test_counter_add_associative(
            a in any::<WcCounter>(),
            b in any::<WcCounter>(),
            c in any::<WcCounter>()
        ) {
            let mut sum1 = a.clone();
            sum1 += &b;
            sum1 += &c;

            let mut sum2 = b.clone();
            sum2 += &c;
            sum2 += &a;

            assert_eq!(sum1.lines, sum2.lines);
            assert_eq!(sum1.words, sum2.words);
            assert_eq!(sum1.bytes, sum2.bytes);
        }
    }
}