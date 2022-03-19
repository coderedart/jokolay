use serde::*;

/// Describes a person who has made markers. used for both pack and category descriptions
/// please consider privacy when you are exposing your details. especially email or real name.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(default)]
pub struct Author {
    /// either Real name or just an Alias
    #[serde(skip_serializing_if = "String::is_empty")]
    pub name: String,
    /// email of the person.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub email: String,
    /// In Game ID. use the account name here. abcd.1234 format. eg: JokoPug.3421
    #[serde(skip_serializing_if = "String::is_empty")]
    pub ign: String,
    /// any other info you would like to provide regarding an author.
    /// any donation links like patreon .
    /// what they contributed to specifically etc..
    #[serde(skip_serializing_if = "String::is_empty")]
    pub extra: String,
}
impl Default for Author {
    fn default() -> Self {
        Self {
            name: "your name".to_string(),
            email: "".to_string(),
            ign: "".to_string(),
            extra: "".to_string(),
        }
    }
}
#[cfg(test)]
mod tests {
    use crate::json::author::Author;
    use serde_test::*;

    #[test]
    fn serde_author() {
        let author = Author {
            name: "me".to_string(),
            email: "me@jokolay.com".to_string(),
            ign: "".to_string(),
            extra: "".to_string(),
        };

        assert_tokens(
            &author,
            &[
                Token::Struct {
                    name: "Author",
                    len: 2,
                },
                Token::Str("name"),
                Token::String("me"),
                Token::Str("email"),
                Token::String("me@jokolay.com"),
                Token::StructEnd,
            ],
        );
    }
}
