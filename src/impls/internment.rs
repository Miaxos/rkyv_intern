use std::{fmt::Debug, hash::Hash, str::FromStr};

use crate::{ArchivedInternedString, Intern, InternSerializeRegistry, InternedStringResolver};
use internment::Intern as InternLocal;
use rkyv::{
    ser::Serializer,
    with::{ArchiveWith, DeserializeWith, SerializeWith},
    Archive, Fallible,
};

/// An Interned structure with Archived where T is Archived and will be serialized as a String
pub type InternedRkyvString<T> = InternLocal<T>;

impl<T: Archive + AsRef<str>> ArchiveWith<InternedRkyvString<T>> for Intern {
    type Archived = ArchivedInternedString;
    type Resolver = InternedStringResolver;

    unsafe fn resolve_with(
        field: &InternedRkyvString<T>,
        pos: usize,
        resolver: Self::Resolver,
        out: *mut Self::Archived,
    ) {
        ArchivedInternedString::resolve_from_str((*(*field)).as_ref(), pos, resolver, out);
    }
}

impl<T: Archive + AsRef<str>, S: InternSerializeRegistry<String> + Serializer + ?Sized>
    SerializeWith<InternedRkyvString<T>, S> for Intern
{
    fn serialize_with(
        field: &InternedRkyvString<T>,
        serializer: &mut S,
    ) -> Result<Self::Resolver, S::Error> {
        ArchivedInternedString::serialize_from_str((*(*field)).as_ref(), serializer)
    }
}

// What is this you ask?
//
// We do internment inside a rkyv format, but when we deserialize it, we put it into a Intern
// struct which create a HashMap, it's not the most efficient method as it performs unecessary
// operations: we do not really need to check the creation of the hashmap from Intern, we could
// create a more efficient version.
impl<
        T: Archive
            + AsRef<str>
            + FromStr<Err = impl Eq + Hash + Sync + Send + Debug>
            + Eq
            + Hash
            + Send
            + Sync
            + 'static,
        D: Fallible + ?Sized,
    > DeserializeWith<ArchivedInternedString, InternedRkyvString<T>, D> for Intern
{
    fn deserialize_with(
        field: &ArchivedInternedString,
        _: &mut D,
    ) -> Result<InternedRkyvString<T>, D::Error> {
        // Remove the unwrap
        Ok(InternLocal::new(T::from_str(field.as_str()).unwrap()))
    }
}
