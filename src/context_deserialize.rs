use crate::{typenum::Unsigned, FixedVector, ProgressiveVariableList};
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

impl<'de, C, T, N> ContextDeserialize<'de, C> for ProgressiveVariableList<T, N>
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
        ProgressiveVariableList::new(vec).map_err(D::Error::custom)
    }
}

#[cfg(test)]
mod test {
    use crate::ProgressiveVariableList;
    use context_deserialize::ContextDeserialize;
    use typenum::U4;

    fn context_deserialize_u64s(json: &str) -> Result<ProgressiveVariableList<u64, U4>, String> {
        let mut deserializer = serde_json::Deserializer::from_str(json);
        ProgressiveVariableList::context_deserialize(&mut deserializer, ())
            .map_err(|e| e.to_string())
    }

    #[test]
    fn context_deserialize_accepts_list_at_limit() {
        let list = context_deserialize_u64s("[1,2,3,4]").unwrap();
        assert_eq!(&list[..], &[1, 2, 3, 4]);
    }

    #[test]
    fn context_deserialize_rejects_list_past_limit() {
        let err = context_deserialize_u64s("[1,2,3,4,5]").unwrap_err();
        assert!(
            err.contains("Index out of bounds: index 5, length 4"),
            "{err}"
        );
    }
}
