pub mod internal;
pub mod json;
pub mod manager;
pub mod xmlpack;

pub const INCHES_PER_METER: f32 = 39.370_08;
pub fn is_default<T: PartialEq + Default>(t: &T) -> bool {
    t == &T::default()
}

// pub(crate) trait PartialEqDefault : PartialEq + Default
// {
//     fn is_default (&self) -> bool;
// }
//
// impl<T : PartialEq + Default> PartialEqDefault for T
// {
//     // default /* with feature specialization */
//     fn is_default (&self) -> bool
//     {
//         Self::default().eq(self)
//     }
// }
