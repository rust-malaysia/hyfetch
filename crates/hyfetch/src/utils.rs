use std::path::PathBuf;

use anyhow::{Context, Result};
use directories::ProjectDirs;

pub fn get_cache_path() -> Result<PathBuf> {
    let path = ProjectDirs::from("", "", "hyfetch")
        .context("failed to get base dirs")?
        .cache_dir()
        .to_owned();
    Ok(path)
}

pub(crate) mod index_map_serde {
    use std::fmt;
    use std::fmt::Display;
    use std::hash::Hash;
    use std::marker::PhantomData;
    use std::str::FromStr;

    use indexmap::IndexMap;
    use serde::de::{self, DeserializeSeed, MapAccess, Visitor};
    use serde::{Deserialize, Deserializer};

    pub(crate) fn deserialize<'de, D, K, V>(deserializer: D) -> Result<IndexMap<K, V>, D::Error>
    where
        D: Deserializer<'de>,
        K: Eq + Hash + FromStr,
        K::Err: Display,
        V: Deserialize<'de>,
    {
        struct KeySeed<K> {
            k: PhantomData<K>,
        }

        impl<'de, K> DeserializeSeed<'de> for KeySeed<K>
        where
            K: FromStr,
            K::Err: Display,
        {
            type Value = K;

            fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: Deserializer<'de>,
            {
                deserializer.deserialize_str(self)
            }
        }

        impl<'de, K> Visitor<'de> for KeySeed<K>
        where
            K: FromStr,
            K::Err: Display,
        {
            type Value = K;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string")
            }

            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                K::from_str(s).map_err(de::Error::custom)
            }
        }

        struct MapVisitor<K, V> {
            k: PhantomData<K>,
            v: PhantomData<V>,
        }

        impl<'de, K, V> Visitor<'de> for MapVisitor<K, V>
        where
            K: Eq + Hash + FromStr,
            K::Err: Display,
            V: Deserialize<'de>,
        {
            type Value = IndexMap<K, V>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a map")
            }

            fn visit_map<A>(self, mut input: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut map = IndexMap::new();
                while let Some((k, v)) =
                    input.next_entry_seed(KeySeed { k: PhantomData }, PhantomData)?
                {
                    map.insert(k, v);
                }
                Ok(map)
            }
        }

        deserializer.deserialize_map(MapVisitor {
            k: PhantomData,
            v: PhantomData,
        })
    }
}
