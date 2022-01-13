use crate::json::Author;
use serde::{Deserialize, Serialize};

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Cat {
    pub name: String,
    pub display_name: String,
    #[serde(default)]
    #[serde(skip_serializing_if = "is_false")]
    pub is_separator: bool,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub authors: Vec<Author>,
}

fn is_false(b: &bool) -> bool {
    !b
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CatTree {
    pub id: u16,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<CatTree>,
}


#[cfg(test)]
mod tests {
    use serde_test::*;

    use crate::json::category::{Cat, CatTree};

    #[test]
    fn serde_cat_description() {
        let cat_desc = Cat {
            name: "marker category one".to_string(),
            display_name: "One".to_string(),
            is_separator: false,
            authors: vec![],
        };

        assert_tokens(
            &cat_desc,
            &[
                Token::Struct{ name: "Cat", len: 2 },
                Token::Str("name"),
                Token::String("marker category one"),
                Token::Str("display_name"),
                Token::String("One"),
                Token::StructEnd
            ],
        );
    }
    #[test]
    fn serde_cat_tree() {
        
        let cat_tree = CatTree {
            id: 1,
            children: vec![CatTree {id: 2, children: vec![]}, CatTree {id: 3, children: vec![]}],
        };

        assert_tokens(
            &cat_tree,
            &[
                Token::Struct{ name: "CatTree", len: 2 },
                Token::Str("id"),
                Token::U16(1),
                Token::Str("children"),
                Token::Seq {len: Some(2)},
                Token::Struct{ name: "CatTree", len: 1 },
                Token::Str("id"),
                Token::U16(2),
                Token::StructEnd,
                Token::Struct{ name: "CatTree", len: 1 },
                Token::Str("id"),
                Token::U16(3),
                Token::StructEnd,
                Token::SeqEnd,
                Token::StructEnd
            ],
        );
    }
}
