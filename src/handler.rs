
use std::collections::{HashMap, HashSet};
use std::ops::ControlFlow;

use crate::config::ZmKeyword;

type BoxedPredicateFn<'cfg> = Box<dyn Fn(&[String], usize) -> bool + 'cfg>;

/// From<F: Fn(...) -> bool> と From<T: AsRef<str>> の実装が衝突するので、後者は実装しない。
pub struct PredicateFn<'cfg>(BoxedPredicateFn<'cfg>);
impl<'cfg> From<&'cfg str> for PredicateFn<'cfg> {
    fn from(value: &'cfg str) -> Self {
        Self(Box::new(move |args, i| args[i] == value))
    }
}
impl<'cfg, F> From<F> for PredicateFn<'cfg>
where
    F: Fn(&[String], usize) -> bool + 'cfg,
{
    fn from(value: F) -> Self {
        Self(Box::new(value))
    }
}


type BoxedReplacerFn<'cfg> = Box<dyn Fn(&[String], usize) -> anyhow::Result<(Vec<String>, usize)> + 'cfg>;

/// From<F: Fn(...) -> anyhow::Result<...>> と From<T: AsRef<str>> の実装が衝突するので、後者は実装しない。
pub struct ReplacerFn<'cfg>(BoxedReplacerFn<'cfg>);
impl<'cfg> From<&'cfg str> for ReplacerFn<'cfg> {
    fn from(value: &'cfg str) -> Self {
        Self(Box::new(move |_, i| Ok((vec![value.into()], i + 1))))
    }
}
impl<'cfg, F> From<F> for ReplacerFn<'cfg>
where
    F: Fn(&[String], usize) -> anyhow::Result<(Vec<String>, usize)> + 'cfg,
{
    fn from(value: F) -> Self {
        Self(Box::new(value))
    }
}

#[derive(Default)]
pub struct HandlerTable<'cfg> {
    registerred: HashSet<String>,
    keywords: HashMap<String, ReplacerFn<'cfg>>,
    handlers: Vec<(PredicateFn<'cfg>, ReplacerFn<'cfg>)>,
}

impl<'cfg> HandlerTable<'cfg> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_word_to_word(&mut self, word: &str, replaced: &'cfg str) -> bool {
        if !self.registerred.contains(word) {
            self.registerred.insert(word.to_string());
            assert!(!self.keywords.contains_key(word));
            self.keywords.insert(word.to_string(), replaced.into());
            return true
        }
        false
    }

    pub fn add_word_to_replacer<R>(&mut self, word: &str, replacer: R) -> bool
    where
        R: Fn(&[String], usize) -> anyhow::Result<(Vec<String>, usize)> + 'cfg,
    {
        if !self.registerred.contains(word) {
            self.registerred.insert(word.to_string());
            assert!(!self.keywords.contains_key(word));
            self.keywords.insert(word.into(), replacer.into());
            return true
        }
        false
    }

    pub fn add_pred_to_word<P>(&mut self, pred: P, replaced: &'cfg str)
    where
        P: Fn(&[String], usize) -> bool + 'cfg,
    {
        self.handlers.push((pred.into(), replaced.into()));
    }
    
    pub fn add_pred_to_replacer<P, R>(&mut self, pred: P, replacer: R)
    where
        P: Fn(&[String], usize) -> bool + 'cfg,
        R: Fn(&[String], usize) -> anyhow::Result<(Vec<String>, usize)> + 'cfg,
    {
        self.handlers.push((pred.into(), replacer.into()));
    }

    pub fn add_keyword(&mut self, keyword: &'cfg ZmKeyword) -> bool {
        match (keyword.prefix.as_ref(), keyword.mapping.as_ref()) {
            (Some(prefix), None) => {
                let pred = move |args: &[String], i: usize| {
                    args[i].starts_with(prefix) &&
                    args[i][prefix.len()..] == keyword.name
                };
                self.add_pred_to_word(pred, &keyword.name);
                true
            },
            (Some(prefix), Some(mapping)) => {
                let conflicted = mapping.keys().map(|k| {
                    if !self.registerred.contains(k) {
                        self.registerred.insert(k.clone());
                        false
                    } else {
                        true
                    }
                }).all(|b| b);
                if conflicted {
                    return false;
                }
                let pred = move |args: &[String], i: usize| {
                    args[i].starts_with(prefix) &&
                    mapping.contains_key(&args[i][prefix.len()..])
                };
                let replace = |args: &[String], i: usize| {
                    Ok((vec![mapping.get(&args[i][prefix.len()..]).unwrap().to_string()], i + 1))
                };
                self.add_pred_to_replacer(pred, replace);
                true
            },
            (None, Some(mapping)) => {
                let conflicted = mapping.keys().map(|k| {
                    if !self.registerred.contains(k) {
                        self.registerred.insert(k.clone());
                        false
                    } else {
                        true
                    }
                }).all(|b| b);
                if conflicted {
                    return false;
                }
                let pred = move |args: &[String], i: usize| {
                    mapping.contains_key(&args[i])
                };
                let replace = |args: &[String], i: usize| {
                    Ok((vec![mapping.get(&args[i]).unwrap().to_string()], i + 1))
                };
                self.add_pred_to_replacer(pred, replace);
                true
            },
            (None, None) => {
                self.add_word_to_word(&keyword.name, &keyword.name)
            }
        }
    }

    pub fn parse_args(&self, args: &[String]) -> anyhow::Result<Vec<String>> {
        let mut newer = vec![];

        let mut i = 0;
        while i < args.len() {
            let arg = &args[i];
            if let Some(replacer) = self.keywords.get(arg) {
                let (replaced, new_pos) = replacer.0(args, i)?;
                newer.extend(replaced.to_vec());
                i = new_pos;
                continue;
            } else {
                let res = self.handlers.iter().try_for_each(|(pred, repl)| {
                    if pred.0(args, i) {
                        return ControlFlow::Break(repl.0(args, i))
                    }
                    ControlFlow::Continue(())
                });
                match res {
                    ControlFlow::Break(res) => {
                        let (replaced, new_i) = res?;
                        newer.extend(replaced);
                        i = new_i;
                    }
                    ControlFlow::Continue(_) => {
                        newer.push(args[i].to_string());
                        i += 1;
                    }
                }
            }
        }

        Ok(newer)
    }
}
