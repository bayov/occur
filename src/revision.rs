//! TODO: Doc

use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;

/// A type whose instances have revisions.
///
/// Documentation for this trait assumes it is implemented for an enum type,
/// and that each enum variant is assigned a unique revision value.
pub trait Revision: Clone {
    /// Used as the revision value that uniquely distinguishes enum variants.
    ///
    /// By default, this is a pair of a string name and revision number. The
    /// name typically matches the enum variant's identifier, and the revision
    /// number starts on 0 then increments by 1 every time a new revision is
    /// introduced for the enum variant.
    type Value: Debug + Clone + Eq + Hash = (&'static str, u8);

    /// Returns the revision value of the enum variant.
    fn revision(&self) -> Self::Value;

    /// Returns the set of all revision values that are represented by the enum.
    fn revision_set() -> HashSet<Self::Value>;
}

/// Holds either an old revision or a new one.
pub enum OldOrNew<OldRevision, NewRevision>
where
    OldRevision: Revision<Value = NewRevision::Value>,
    NewRevision: Revision,
{
    Old(OldRevision),
    New(NewRevision),
}

/// A type whose instances can be converted to newer revisions of themselves.
pub trait Convert: Revision {
    /// The newer type to which this type can be converted to.
    type New: Revision<Value = Self::Value>;

    /// Converts this variant instance a newer one.
    ///
    /// Use [`Self::convert_until_new`] to convert an old variant as many times
    /// as needed to acquire an instance of [`Self::New`].
    ///
    /// Ensure that each invocation of `convert` returns a newer variant
    /// revision, to avoid an infinite conversion loop.
    fn convert(self) -> OldOrNew<Self, Self::New>;

    /// Converts this instances as many times as needed until it becomes a new
    /// variant type.
    fn convert_until_new(self) -> Self::New {
        match Self::convert(self) {
            OldOrNew::Old(old) => old.convert_until_new(),
            OldOrNew::New(new) => new,
        }
    }
}

/// Represents a revision with no variants.
///
/// Used as the default [`crate::Event::OldRevision`] type, indicating that
/// there are no old revision variants for the event.
pub struct Empty<T: Revision>(!, PhantomData<T>);

impl<T: Revision> Clone for Empty<T> {
    fn clone(&self) -> Self { unreachable!() }
}

impl<T: Revision> Revision for Empty<T> {
    type Value = T::Value;
    fn revision(&self) -> Self::Value { unreachable!() }
    fn revision_set() -> HashSet<Self::Value> { HashSet::default() }
}

impl<T: Revision> Convert for Empty<T> {
    type New = T;
    fn convert(self) -> OldOrNew<Self, Self::New> { unreachable!() }
}
