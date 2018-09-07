use cssparser;

use super::types::*;

impl Stylesheet {
    fn parse(stylesheet: &str) -> Stylesheet {
        Stylesheet { rules: vec![] }
    }
}
