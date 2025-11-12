use crate::RuntimeVariableList;
use context_deserialize::ContextDeserialize;
use serde::{de::Error as DeError, Deserializer};

impl<'de, C, T> ContextDeserialize<'de, (C, usize)> for RuntimeVariableList<T>
where
    T: ContextDeserialize<'de, C>,
    C: Clone,
{
    fn context_deserialize<D>(deserializer: D, context: (C, usize)) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // First parse out a Vec<C> using the Vec<C> impl you already have.
        let vec: Vec<T> = Vec::context_deserialize(deserializer, context.0)?;
        let vec_len = vec.len();
        RuntimeVariableList::new(vec, context.1).map_err(|e| {
            DeError::custom(format!(
                "RuntimeVariableList length {} exceeds max_len {}: {e:?}",
                vec_len, context.1,
            ))
        })
    }
}
