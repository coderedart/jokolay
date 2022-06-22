//! Categories need to be made as easy to edit as possible.
//! MarkerCategory tags of xml packs lead to three issues.
//! example:
//! ```xml
//!
//! <MarkerCategory name="parent" displayName="parent name" isSeparator="0" default_toggle="1" >
//!   <MarkerCategory name="child" displayName="child name" >
//!     <MarkerCategory name="subchild" displayName="SubChild" />
//!   </MarkerCategory>
//! </MarkerCategory>
//!
//! ```
//!
//! 1. markers need to refer to the categories by xpath "parent.child.subchild". so
//!     if we change the menu relationship (move categories around), ALL markers that refer to
//!     the changed categories need to change their category xpath dealing with such strings
//!     will also be a mess regardless of other issues that come with string attributes like
//!     case sensitivity.
//! 2. categories are spread across different xml files. very error prone. for example, the
//!     other file may have child.parent.subchild order instead of the above parent.child.subchild
//! 3. marker categories have attributes that are inherited, which makes editing complicated.
//!     it makes dealing with the whole markers complicated. you need to search around in the
//!     whole chain of inheritance from where a particular attribute is affecting the marker.
//!
//! to make json categories simply we will tackle the above issues one by one.
//! 1. we will add a id field to category to uniquely identify it. and markers can refer to that
//!     category using the id. so, even if the category itself is moved around inside the menu
//!     tree, its id will still be the same. the id will be a u16 for now.
//! 2. there will be a cats.json file which will contain only categories and there will only be one
//!     cats.json file per pack.
//! 3. categories will not have any inheritable attributes at all. they only need:
//!     id, display name for menu, default_toggle for first time pack menu selection, is_separator.
//!
//! now, categories can be edited separately without affecting markers.
//!     

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// The Category struct that stores its own info and none of the marker/trail attributes.
/// we skip the name attribute as we will just use the id as its name for xpath in xml packs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Category {
    id: u16,
    pub display_name: String,
    pub is_separator: bool,
    pub default_toggle: bool,
    children: Vec<Category>,
}
impl Category {
    fn new(new_id: u16) -> Self {
        Self {
            id: new_id,
            display_name: "New Category".to_string(),
            is_separator: false,
            default_toggle: true,
            children: vec![],
        }
    }
    pub fn get_id(&self) -> u16 {
        self.id
    }
    pub fn get_children(&self) -> &[Category] {
        self.children.as_slice()
    }
    pub fn get_children_mut(&mut self) -> &mut [Self] {
        self.children.as_mut_slice()
    }
    fn add_child(&mut self, new_id: u16) -> &mut Self {
        self.children.push(Self::new(new_id));
        self.children
            .last_mut()
            .expect("unreachable because we just added a category")
    }

    fn recurse_get_cat_mut(categories: &mut [Category], category_id: u16) -> Option<&mut Self> {
        for cat in categories {
            if cat.id == category_id {
                return Some(cat);
            }
            if let Some(child) = Self::recurse_get_cat_mut(&mut cat.children, category_id) {
                return Some(child);
            }
        }
        None
    }
    fn recurse_get_cat(categories: &[Category], category_id: u16) -> Option<&Self> {
        for cat in categories {
            if cat.id == category_id {
                return Some(cat);
            }
            if let Some(child) = Self::recurse_get_cat(&cat.children, category_id) {
                return Some(child);
            }
        }
        None
    }
    /// This will recursively go through a slice of `Category` and remove their ids from the
    /// `remaining_ids` set. if the set doesn't contain that ID it means there's a duplicate ID somewhere.
    /// so, we will panic as that must never happen.
    ///
    /// Arguments:
    /// * cats: a slice of categories for the function to get the used ids from.
    /// * remaining_ids: A set which contains *atleast* all ids which are used in `cats` slice or their children.
    fn recurse_remove_used_ids(cats: &[Category], remaining_ids: &mut BTreeSet<u16>) {
        for cat in cats {
            assert!(remaining_ids.remove(&cat.id));
            Self::recurse_remove_used_ids(&cat.children, remaining_ids);
        }
    }
}

/// The category menu is the struct that keeps the categories. this will encapsulate the category struct
/// it must keep its fields private to ensure that it will maintain its invariants
/// like no duplicate ids inside categories
/// Ideally, all external api should work with this and its fields should not be exposed
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct CategoryMenu {
    cats: Vec<Category>,
}

impl CategoryMenu {
    pub fn get_category(&self, category_id: u16) -> Option<&Category> {
        Category::recurse_get_cat(&self.cats, category_id)
    }
    pub fn get_category_mut(&mut self, category_id: u16) -> Option<&mut Category> {
        Category::recurse_get_cat_mut(&mut self.cats, category_id)
    }
    /// checks if there is a Category inside the menu with the given ID
    pub fn does_id_exist(&self, id_to_check: u16) -> bool {
        Category::recurse_get_cat(&self.cats, id_to_check).is_some()
    }

    /// This will recurse through the cats contained within the `CategoryMenu` and
    /// find a id which is unused. this `u16` id can be used for a new category which
    /// needs to be inserted into the `CategoryMenu`
    fn get_unused_id(&self) -> u16 {
        // contains all possible IDS
        let mut all_ids: BTreeSet<u16> = (0..=u16::MAX).collect();
        // removes all existing ids
        Category::recurse_remove_used_ids(&self.cats, &mut all_ids);

        // only ids which are not used are contained in the set now
        all_ids
            .into_iter()
            .take(1) // take the first id that is not used
            .next()
            .expect("cannot get unused category ID because its full") // will probably won't happen anytime soon
    }

    /// Create a new category at the end of the children of a category with id `parent_id`
    /// we use a new id which is the smallest unused u16 in the CategoryMenu
    /// panics if there's no category with id `parent_id`
    /// Arguments:
    /// * parent_id: if this is None, we will create a category at root without any parent.
    pub fn create_child_category(&mut self, parent_id: Option<u16>) -> &mut Category {
        let new_id = self.get_unused_id();
        if let Some(parent_id) = parent_id {
            Category::recurse_get_cat_mut(&mut self.cats, parent_id)
                .expect("failed to get parent category to insert child")
                .add_child(new_id)
        } else {
            self.cats.push(Category::new(new_id));
            self.cats
                .last_mut()
                .expect("just inserted category. unreachable")
        }
    }
}

#[cfg(test)]
mod test {

    use rstest::*;
    use similar_asserts::assert_eq;

    use super::{Category, CategoryMenu};

    #[fixture]
    fn category_menu() -> CategoryMenu {
        CategoryMenu {
            cats: vec![Category {
                id: 3,
                display_name: "Third".to_string(),
                is_separator: false,
                default_toggle: true,
                children: vec![Category {
                    id: 4,
                    display_name: "Fourth".to_string(),
                    is_separator: false,
                    default_toggle: true,
                    children: vec![],
                }],
            }],
        }
    }
    #[fixture]
    fn full_category_menu() -> CategoryMenu {
        CategoryMenu {
            cats: (0..=u16::MAX)
                .map(|new_id| Category {
                    id: new_id,
                    display_name: "".to_string(),
                    is_separator: false,
                    default_toggle: false,
                    children: vec![],
                })
                .collect(),
        }
    }
    #[rstest]
    fn simple_equality_test(category_menu: CategoryMenu) {
        assert_eq!(category_menu, category_menu);
    }

    /// This function primarily inserts a bunch of categories and checks that the first unused id
    /// is the lowest unused u16 in the Category menu
    #[rstest]
    #[case(0, 0)]
    #[case(1, 1)]
    #[case(2, 2)]
    #[case(3, 5)]
    fn check_unused_id(
        mut category_menu: CategoryMenu,
        #[case] number_of_new_categories_to_insert: u16,
        #[case] first_unused_id: u16,
    ) {
        for _ in 0..number_of_new_categories_to_insert {
            category_menu.create_child_category(None);
        }
        assert_eq!(first_unused_id, category_menu.get_unused_id());
    }

    /// test that running out of new unused ids due to number of categories
    /// exceeding u16::MAX panics
    #[rstest]
    #[should_panic]
    fn panic_if_max_number_of_categories_reached(full_category_menu: CategoryMenu) {
        assert_eq!(full_category_menu.cats.len() + 1, u16::MAX as usize);
        full_category_menu.get_unused_id();
    }
}
