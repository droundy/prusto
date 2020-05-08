use std::fmt;
use std::marker::PhantomData;

use serde::de::{self, DeserializeSeed, Deserializer, SeqAccess, Visitor};
use serde::Serialize;

use super::util::SerializeIterator;
use super::{Error, Presto, PrestoTy};

impl<T: Presto> Presto for Vec<T> {
    type ValueType<'a> = impl Serialize;
    type Seed<'a, 'de> = VecSeed<'a, T>;

    fn value(&self) -> Self::ValueType<'_> {
        let iter = self.iter().map(|t| t.value());

        SerializeIterator {
            iter,
            size: Some(self.len()),
        }
    }

    fn ty() -> PrestoTy {
        PrestoTy::Array(Box::new(T::ty()))
    }

    fn seed<'a, 'de>(ty: &'a PrestoTy) -> Result<Self::Seed<'a, 'de>, Error> {
        if let PrestoTy::Array(ty) = ty {
            Ok(VecSeed(ty, PhantomData))
        } else {
            Err(Error::InvalidPrestoType)
        }
    }
}

pub struct VecSeed<'a, T>(pub(super) &'a PrestoTy, pub(super) PhantomData<T>);

impl<'a, 'de, T: Presto> Visitor<'de> for VecSeed<'a, T> {
    type Value = Vec<T>;
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("vec seed")
    }
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut ret = vec![];
        while let Some(d) = seq.next_element_seed(
            T::seed(self.0).map_err(|e| <A::Error as de::Error>::custom(format!("{}", e)))?,
        )? {
            ret.push(d)
        }
        Ok(ret)
    }
}

impl<'a, 'de, T: Presto> DeserializeSeed<'de> for VecSeed<'a, T> {
    type Value = Vec<T>;
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(self)
    }
}
