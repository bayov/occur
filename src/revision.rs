//! TODO: Doc

use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;

use crate::Event;

/// A type whose instances have revisions.
///
/// Documentation for this trait assumes it is implemented for an enum type,
/// and that each enum variant is assigned a unique revision value.
pub trait Revision: Clone + Send + Sync {
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

/// Holds either a new event or an old revision of one.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum OldOrNew<T: Event, Old = <T as Event>::OldRevision> {
    Old(Old),
    New(T),
}

impl<T: Event> From<T> for OldOrNew<T> {
    fn from(new_event: T) -> Self { Self::New(new_event) }
}

impl<T: Event> OldOrNew<T> {
    /// Converts to a new revision variant.
    pub fn to_new(self) -> T {
        match self {
            Self::Old(old) => old.convert_until_new(),
            Self::New(new) => new,
        }
    }

    #[must_use]
    pub const fn borrow(&self) -> OldOrNewRef<'_, T> {
        match self {
            Self::Old(old) => OldOrNewRef::Old(old),
            Self::New(new) => OldOrNewRef::New(new),
        }
    }
}

/// Holds either a reference to a new event or an old revision of one.
#[derive(Eq, PartialEq, Debug)]
pub enum OldOrNewRef<'a, T: Event> {
    Old(&'a T::OldRevision),
    New(&'a T),
}

impl<'a, T: Event> OldOrNewRef<'a, T> {
    #[must_use]
    pub fn to_owned(self) -> OldOrNew<T> {
        match self {
            Self::Old(old) => OldOrNew::Old(old.to_owned()),
            Self::New(new) => OldOrNew::New(new.to_owned()),
        }
    }
}

/// A type whose instances can be converted to newer revisions of themselves.
pub trait Convert: Revision {
    /// The newer type to which this type can be converted to.
    type Event: Event<Value = Self::Value>;

    /// Converts this variant instance a newer one.
    ///
    /// Use [`Self::convert_until_new`] to convert an old variant as many times
    /// as needed to acquire an instance of [`Self::New`].
    ///
    /// Ensure that each invocation of `convert` returns a newer variant
    /// revision, to avoid an infinite conversion loop.
    fn convert(self) -> OldOrNew<Self::Event>;

    /// Converts this instances as many times as needed until it becomes a new
    /// variant type.
    fn convert_until_new(self) -> Self::Event {
        match Self::convert(self) {
            OldOrNew::Old(old) => old.convert_until_new(),
            OldOrNew::New(new) => new,
        }
    }
}

/// Represents a revision with no variants.
///
/// Used as the default [`Event::OldRevision`] type, indicating that
/// there are no old revision variants for the event.
pub struct Empty<T: Event>(!, PhantomData<T>);

impl<T: Event> Clone for Empty<T> {
    fn clone(&self) -> Self { unreachable!() }
}

impl<T: Event> Revision for Empty<T> {
    type Value = T::Value;
    fn revision(&self) -> Self::Value { unreachable!() }
    fn revision_set() -> HashSet<Self::Value> { HashSet::default() }
}

impl<T: Event> Convert for Empty<T> {
    type Event = T;
    fn convert(self) -> OldOrNew<Self::Event> { unreachable!() }
}
