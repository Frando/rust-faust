#![allow(clippy::module_name_repetitions)]

use crate::{
    code_option::{CodeOption, CodeOptionDiscriminants},
    FaustArgsToCommandArgs, FaustArgsToCommandArgsRef,
};
use std::{
    cmp::Eq,
    collections::{hash_map::IntoValues, HashMap, HashSet},
    ffi::OsStr,
    hash::Hash,
    iter::FromIterator,
};
use strum::IntoDiscriminant;

#[derive(Debug, Clone)]
pub struct OptionMap<K, V>(HashMap<K, V>);

pub type CodeOptionMap = OptionMap<CodeOptionDiscriminants, CodeOption>;

impl<'a, K, V> OptionMap<K, V>
where
    K: Eq + Hash,
    V: FaustArgsToCommandArgsRef<'a> + IntoDiscriminant<Discriminant = K>,
{
    pub fn insert(&mut self, value: V) -> Option<V> {
        self.0.insert(V::discriminant(&value), value)
    }

    #[must_use]
    pub fn get(&self, key: &K) -> Option<&V> {
        self.0.get(key)
    }

    #[must_use]
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    #[must_use]
    pub fn to_command_args_merge(&'a self, other_args: &'a Self) -> Vec<&'a OsStr> {
        let keys: HashSet<&K> = self.0.keys().chain(other_args.0.keys()).collect();
        let values = keys
            .iter()
            .map(|key| {
                other_args
                    .get(key)
                    .unwrap_or_else(|| self.get(key).unwrap())
            })
            .collect::<Vec<_>>();

        FaustArgsToCommandArgs::to_command_args(values)
    }
}

impl<'a, K, V> Extend<V> for OptionMap<K, V>
where
    K: Eq + Hash,
    V: FaustArgsToCommandArgsRef<'a> + IntoDiscriminant<Discriminant = K>,
{
    fn extend<T: IntoIterator<Item = V>>(&mut self, iter: T) {
        for i in iter {
            self.insert(i);
        }
    }
}

impl<'a, K, V> FromIterator<V> for OptionMap<K, V>
where
    K: Eq + Hash,
    V: FaustArgsToCommandArgsRef<'a> + IntoDiscriminant<Discriminant = K>,
{
    fn from_iter<T: IntoIterator<Item = V>>(iter: T) -> Self {
        let mut r = Self(HashMap::new());
        for i in iter {
            r.insert(i);
        }
        r
    }
}

impl<K, V> IntoIterator for OptionMap<K, V> {
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_values()
    }
    type Item = V;

    type IntoIter = IntoValues<K, V>;
}

impl Default for CodeOptionMap {
    fn default() -> Self {
        Self(HashMap::default())
    }
}

impl<'a, K, V: FaustArgsToCommandArgsRef<'a>> FaustArgsToCommandArgs<'a> for &'a OptionMap<K, V> {
    fn to_command_args(self) -> Vec<&'a OsStr> {
        FaustArgsToCommandArgs::to_command_args(self.0.values())
    }
}
