use std::path::{Path, PathBuf};

use super::prelude::*;
use std::ffi::OsStr;
use std::collections::HashMap;
use std::hash::Hash;

pub trait StrExtensions {
    fn last_index_of(&self, c: char) -> Option<usize>;
}

impl StrExtensions for &str {
    fn last_index_of(&self, c: char) -> Option<usize> {

        let mut i = self.len() - 1;

        for x in self.chars().rev() {

            if x == c {
                return Some(i);
            }

            if i > 0 {
                i -= 1;
            }
        }

        None
    }
}

pub trait OsStrExtensions {
    fn get_as_string(&self) -> Result<String>;
}

impl OsStrExtensions for OsStr {
    fn get_as_string(&self) -> Result<String> {

        Ok(self.to_str()
            .ok_or_else(|| CustomError::from_message("The OsStr cannot be converted to &str because it is not valid."))
            ?.to_string())
    }
}

pub trait PathExtensions {
    fn get_as_string(&self) -> Result<String>;
    fn extension_as_string(&self) -> Result<String>;
    fn file_stem_as_string(&self) -> Result<String>;
    fn file_name_as_string(&self) -> Result<String>;
    fn get_directory_as_string(&self) -> Result<String>;
    fn get_directory(&self) -> PathBuf;
    fn combine_with(&self, p: &str) -> PathBuf;
    fn create_directory(&self) -> Result<PathBuf>;
}

impl PathExtensions for Path {

    fn get_as_string(&self) -> Result<String> {
        Ok(self.to_str()
            .ok_or_else(|| CustomError::from_message("The Path cannot be converted to &str because it is not valid."))?
            .to_string())
    }

    fn extension_as_string(&self) -> Result<String> {

        Ok(self.extension()
            .ok_or_else(|| CustomError::from_message(
                "The file does not have an extension"
            ))?.get_as_string()?)
    }

    fn file_stem_as_string(&self) -> Result<String> {

        Ok(self.file_stem()
            .ok_or_else(|| CustomError::from_message(
                "The file does not have a `file_stem`."
            ))?.get_as_string()?)
    }

    fn file_name_as_string(&self) -> Result<String> {

        Ok(self.file_name()
            .ok_or_else(|| CustomError::from_message(
                "The file does not have a `file_stem`."
            ))?.get_as_string()?)
    }

    fn get_directory_as_string(&self) -> Result<String> {

        let mut copy = self.to_path_buf();

        copy.pop();

        copy.get_as_string()
    }

    fn get_directory(&self) -> PathBuf {

        let mut copy = self.to_path_buf();

        copy.pop();

        copy
    }

    fn combine_with(&self, p: &str) -> PathBuf {

        let mut copy = self.to_path_buf();

        copy.push(p);

        copy
    }

    fn create_directory(&self) -> Result<PathBuf> {

        let copy = self.to_path_buf();

        ::std::fs::create_dir_all(copy.get_as_string()?)?;

        Ok(copy)
    }
}

pub trait OptionFlatten<T> {
    fn flatten(self) -> Option<T>;
}

impl<T> OptionFlatten<T> for Option<Option<T>> {
    fn flatten(self) -> Option<T> {
        match self {
            None => None,
            Some(v) => v,
        }
    }
}

impl<T> OptionFlatten<T> for Option<Option<Option<T>>> {
    fn flatten(self) -> Option<T> {
        match self {
            None => None,
            Some(v) => v.flatten(),
        }
    }
}

impl<T> OptionFlatten<T> for Option<Option<Option<Option<T>>>> {
    fn flatten(self) -> Option<T> {
        match self {
            None => None,
            Some(v) => v.flatten(),
        }
    }
}

impl<T> OptionFlatten<T> for Option<Option<Option<Option<Option<T>>>>> {
    fn flatten(self) -> Option<T> {
        match self {
            None => None,
            Some(v) => v.flatten(),
        }
    }
}

pub trait OptionBorrow<T> {
    fn map<U, F: FnOnce(&T) -> U>(&self, f: F) -> Option<U>;
    fn map_result<U, F: FnOnce(&T) -> Result<U>>(&self, f: F) -> Result<Option<U>>;
}

impl<T> OptionBorrow<T> for Option<T> {

    fn map<U, F: FnOnce(&T) -> U>(&self, f: F) -> Option<U> {
        match self {
            Some(x) => Some(f(x)),
            None => None,
        }
    }

    fn map_result<U, F: FnOnce(&T) -> Result<U>>(&self, f: F) -> Result<Option<U>> {
        Ok(match self {
            Some(x) => Some(f(x)?),
            None => None,
        })
    }
}

pub trait IteratorExtensions: Iterator {

    fn order_by<K, F>(self, f: F) -> ::std::vec::IntoIter<Self::Item>
        where Self: Sized, K: Ord, F: FnMut(&Self::Item) -> K {

        let mut vec: Vec<Self::Item> = self.collect();
        vec.sort_by_key(f);
        vec.into_iter()
    }

    fn order_by_desc<K, F>(self, f: F) -> ::std::vec::IntoIter<Self::Item>
        where Self: Sized, K: Ord, F: FnMut(&Self::Item) -> K {

        let mut vec: Vec<Self::Item> = self.collect();
        vec.sort_by_key(f);
        vec.reverse();
        vec.into_iter()
    }

    fn group_by<K, F>(self, f: F) -> ::std::collections::hash_map::IntoIter<K, Vec<Self::Item>>
        where Self: Sized, K: Eq, K: Hash, F: Fn(&Self::Item) -> K {

        let mut group_map = HashMap::new();

        for item in self {

            let value = group_map
                .entry(f(&item))
                .or_insert(Vec::new());

            value.push(item);
        }

        group_map.into_iter()
    }

    fn first<F>(self, f: F) -> Option<Self::Item>
        where Self: Sized, Self::Item: Clone, F: Fn(&Self::Item) -> bool {

        let vec: Vec<Self::Item> = self.filter(f).take(1).collect();

        vec.first().map(|x| x.clone())
    }

    fn any_result<F>(self, f: F) -> Result<bool>
        where Self: Sized, F: Fn(&Self::Item) -> Result<bool> {

        for item in self {

            if f(&item)? {
                return Ok(true);
            }
        }

        Ok(false)
    }

    fn map_result<K, F>(self, f: F) -> Result<::std::vec::IntoIter<K>>
        where Self: Sized, F: Fn(&Self::Item) -> Result<K> {

        let source: Vec<Self::Item> = self.collect();

        let mut destination = Vec::new();

        for item in source {

            destination.push(f(&item)?);
        }

        Ok(destination.into_iter())
    }

    fn collect_vec(self) -> Vec<Self::Item> where Self: Sized {

        self.collect()
    }
}

impl<T> IteratorExtensions for T where T: Iterator { }
