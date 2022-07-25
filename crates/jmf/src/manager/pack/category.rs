//!
//! In XML Packs, we have the following issues:
//! 1. Categories are spread across different xml files. very error prone. for example, the
//!     other file may have child.parent.subchild order instead of the above parent.child.subchild
//! 2. Categories have attributes that are inherited, which makes editing complicated.
//!     it makes dealing with the whole markers complicated. you need to search around in the
//!     whole chain of inheritance from where a particular attribute is affecting the marker.
//! 3. Markers / Trails refer to categories using a `parent.child.sub_child` format. it makes
//!     us write custom functions to manipulate that category path.
//!
//! To fix the above issues, we will make the following changes:
//! 1. there will be a cats.json file which will contain only categories (in a tree structure) and there
//!     will only be one cats.json file per pack.
//! 2. categories will not have any inheritable attributes at all. they only need:
//!     name, display name for menu, default_toggle for first time pack menu selection, is_separator.
//! 3. Markers / Trails will refer to categories using `parent/child/sub_child` format. this allows
//!     us to use the existing `std::path::Path` module for path manipulation and much more. ofcourse,
//!     we can also use `crates.io` ecosystem to deal with these now.
//!
//! Category is basically a node with the following properties:
//! 1. name: used as an ID.
//!     not visible to users.
//!     must be unique within a level (vec) of categories.
//!     cannot be empty.
//! 2. display_name: used as the visible ID for category menu.
//! 3. is_separator: marks that this is not a category, but just a label.
//!     cannot have children.
//!     cannot have any "enabled" or "disabled" toggle status
//!     used to divide sections of categories in a certain menu level.
//!     used as "heading" for a series of categories
//! 4. default_toggle: whether a category is enabled when it is first imported by `Jokolay`.
//!     
//! 5. children: list of Categories.
//!     

use camino::Utf8Path;
use serde::{Deserialize, Serialize};

/// The Category struct that stores its own info and none of the marker/trail attributes.
/// we skip the name attribute as we will just use the id as its name for xpath in xml packs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Category {
    pub name: String,
    pub display_name: String,
    pub is_separator: bool,
    pub default_toggle: bool,
    pub children: Vec<Category>,
}

impl Category {
    pub fn new(name: String) -> Self {
        Self {
            name: name.to_lowercase(),
            display_name: name,
            is_separator: false,
            default_toggle: true,
            children: vec![],
        }
    }
}

/// The category menu is the struct that keeps the categories. this will encapsulate the category struct
/// it must keep its fields private to ensure that it will maintain its invariants
/// Ideally, all external api should work with this and its fields should not be exposed
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct CategoryMenu {
    cats: Vec<Category>,
}

impl CategoryMenu {
    /// this will traverse the path given to create the category and will create any parent categories if missing.
    /// once this function runs, we can be sure that the given path exists.
    pub fn create_category(&mut self, path: &Utf8Path) {
        let mut root = &mut self.cats;
        for name in path {
            root = match root.iter_mut().position(|c| c.name == name) {
                Some(c) => &mut root[c],
                None => {
                    root.push(Category::new(name.to_string()));
                    root.last_mut()
                        .expect("just pushed new category. must exist")
                }
            }
            .children
            .as_mut();
        }
    }

    /// removes the category (including its children) if it exists.
    /// cannot delete root, so providing "" (empty) as path will just do nothing
    pub fn delete_category(&mut self, path: &Utf8Path) {
        // if root, just return
        if path.as_str() == "" {
            return;
        }

        let mut root = &mut self.cats;
        // if parent is not root, then we just traverse tree until we reach the parent of the category and assign it to root
        // if parent is root, we will just use the root directly as `path.parent()` returns `None`
        if let Some(parent) = path.parent() {
            for name in parent {
                root = &mut match root.iter_mut().find(|c| c.name == name) {
                    Some(c) => c,
                    None => return, // short circuit. if we miss any node while traversing the path, we can just return as the category doesn't exist.
                }
                .children;
            }
        };
        root.retain(|c| !path.ends_with(&c.name));
    }
    pub fn has_category(&self, path: &Utf8Path) -> bool {
        let mut root = &self.cats;
        for name in path {
            root = &match root.iter().position(|c| c.name == name) {
                Some(index) => &root[index],
                None => return false,
            }
            .children;
        }
        true
    }
    fn get_category<'a>(cats: &'a [Category], path: &Utf8Path) -> Option<&'a Category> {
        path.components() // get components
            .next() // if there's no components, we return None
            .map(|base| {
                // if there's a base component, we will check the current level of categories for a cat with base component as its name
                cats.iter()
                    .position(|c| c.name == base.as_str()) // if there's no category with base componenet as its name, we return None
                    .map(|cat_index| {
                        // but if there is a category
                        match path
                            .strip_prefix(base) // we strip the path of the base
                            .ok() // and check if there's any more remaining path components.
                            .map(|remaining_path| {
                                Self::get_category(&cats[cat_index].children, remaining_path)
                            }) {
                            Some(c) => return c, // if the path still had elements, we would get child category and return that
                            None => return Some(&cats[cat_index]), // if the path was empty, we will return the current cat
                        }
                    })
                    .flatten()
            })
            .flatten()
    }
    fn get_category_mut<'a>(cats: &'a mut [Category], path: &Utf8Path) -> Option<&'a mut Category> {
        let mut children = cats;
        if let Some(parent) = path.parent() {
            for path_node in parent.components() {
                children = match children.iter().position(|c| c.name == path_node.as_str()) {
                    Some(index) => &mut children[index].children,
                    None => break,
                }
            }
        }
        path.file_name()
            .map(|name| children.into_iter().find(|c| c.name == name))
            .flatten()
    }
    pub fn set_name(&mut self, path: &Utf8Path, name: &str) {
        Self::get_category_mut(&mut self.cats, path).map(|cat| cat.name = name.to_lowercase());
    }
    pub fn set_display_name(&mut self, path: &Utf8Path, display_name: String) {
        Self::get_category_mut(&mut self.cats, path)
            .map(|cat| cat.display_name = display_name.to_string());
    }
    pub fn set_default_toggle(&mut self, path: &Utf8Path, toggle: bool) {
        Self::get_category_mut(&mut self.cats, path).map(|cat| cat.default_toggle = toggle);
    }
    pub fn set_is_separator(&mut self, path: &Utf8Path, is_separator: bool) {
        Self::get_category_mut(&mut self.cats, path).map(|cat| cat.is_separator = is_separator);
    }
    pub fn get_name(&self, path: &Utf8Path) -> Option<&str> {
        Some(&Self::get_category(&self.cats, path)?.name)
    }
}

#[cfg(test)]
mod test {

    use camino::Utf8Path;
    use rstest::*;
    use similar_asserts::assert_eq;

    use super::{Category, CategoryMenu};
    /// use as the default category argument for tests
    #[fixture]
    fn category_menu() -> CategoryMenu {
        CategoryMenu {
            cats: vec![
                Category {
                    name: "first".to_string(),
                    display_name: "Second".to_string(),
                    is_separator: false,
                    default_toggle: false,
                    children: Vec::new(),
                },
                Category {
                    name: "third".to_string(),
                    display_name: "Third".to_string(),
                    is_separator: false,
                    default_toggle: true,
                    children: vec![Category {
                        name: "fourth".to_string(),
                        display_name: "Fourth".to_string(),
                        is_separator: false,
                        default_toggle: true,
                        children: vec![],
                    }],
                },
            ],
        }
    }

    #[rstest]
    fn simple_equality_test(category_menu: CategoryMenu) {
        assert_eq!(category_menu, category_menu);
    }
    #[rstest]
    fn create_category(mut category_menu: CategoryMenu) {
        let second = Utf8Path::new("first/second");
        category_menu.create_category(second);
        assert!(category_menu.has_category(second));
    }
    #[rstest]
    fn delete_category(mut category_menu: CategoryMenu) {
        let third_path = Utf8Path::new("third");
        assert!(category_menu.has_category(third_path));
        category_menu.delete_category(third_path);
        assert!(!category_menu.has_category(third_path));
    }
}
