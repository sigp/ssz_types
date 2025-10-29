use crate::{typenum::Unsigned, FixedVector};
use context_deserialize::ContextDeserialize;
use serde::de::{Deserializer, Error};

impl<'de, C, T, N> ContextDeserialize<'de, C> for FixedVector<T, N>
where
    T: ContextDeserialize<'de, C>,
    N: Unsigned,
    C: Clone,
{
    fn context_deserialize<D>(deserializer: D, context: C) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let vec = Vec::<T>::context_deserialize(deserializer, context)?;
        FixedVector::new(vec).map_err(|e| D::Error::custom(format!("{:?}", e)))
    }
}
