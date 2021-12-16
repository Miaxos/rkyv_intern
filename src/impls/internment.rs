use std::{fmt::Debug, hash::Hash, str::FromStr};

use crate::{ArchivedInternedString, Intern, InternSerializeRegistry, InternedStringResolver};
use internment::Intern as InternLocal;
use rkyv::{
    ser::Serializer,
    string::{ArchivedString, StringResolver},
    with::{ArchiveWith, DeserializeWith, SerializeWith},
    Archive, Fallible,
};

/// An Interned structure with Archived where T is Archived and will be serialized as a String
pub struct InternedRkyvString<T: 'static>(pub InternLocal<T>);

impl<T> AsRef<T> for InternedRkyvString<T> {
    fn as_ref(&self) -> &T {
        self.0.as_ref()
    }
}

///An archived [`InernedString`]
#[repr(C)]
#[derive(Hash, PartialEq, Eq)]
pub struct ArchivedRkyvInternedString(
    ///The archived counterpart of [`UserUID::0`]
    rkyv::Archived<String>,
)
where
    String: rkyv::Archive;

pub struct InternedResolver(rkyv::Resolver<String>)
where
    String: rkyv::Archive;

const _: () = {
    impl<T: Archive<Resolver = StringResolver, Archived = ArchivedString>> Archive
        for InternedRkyvString<T>
    where
        String: rkyv::Archive,
    {
        type Archived = ArchivedRkyvInternedString;
        type Resolver = InternedResolver;

        #[allow(clippy::unit_arg)]
        #[inline]
        unsafe fn resolve(&self, pos: usize, resolver: Self::Resolver, out: *mut Self::Archived) {
            let (fp, fo) = {
                #[allow(unused_unsafe)]
                unsafe {
                    let fo = &raw mut (*out).0;
                    (fo.cast::<u8>().offset_from(out.cast::<u8>()) as usize, fo)
                }
            };
            (*self.0).resolve(pos + fp, resolver.0, fo);
        }
    }
};

impl<T: Archive + AsRef<str>> ArchiveWith<InternedRkyvString<T>> for Intern {
    type Archived = ArchivedInternedString;
    type Resolver = InternedStringResolver;

    unsafe fn resolve_with(
        field: &InternedRkyvString<T>,
        pos: usize,
        resolver: Self::Resolver,
        out: *mut Self::Archived,
    ) {
        ArchivedInternedString::resolve_from_str((*field.0).as_ref(), pos, resolver, out);
    }
}

impl<T: Archive + AsRef<str>, S: InternSerializeRegistry<String> + Serializer + ?Sized>
    SerializeWith<InternedRkyvString<T>, S> for Intern
{
    fn serialize_with(
        field: &InternedRkyvString<T>,
        serializer: &mut S,
    ) -> Result<Self::Resolver, S::Error> {
        ArchivedInternedString::serialize_from_str((*field.0).as_ref(), serializer)
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
        Ok(InternedRkyvString(InternLocal::new(
            T::from_str(field.as_str()).unwrap(),
        )))
    }
}
