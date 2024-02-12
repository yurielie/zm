use std::collections::HashMap;
use std::ops::ControlFlow;

use crate::config::ZmKeyword;

type BoxedPredicateFn<'cfg> = Box<dyn Fn(&[String], usize) -> bool + 'cfg>;

// From<F: Fn(...) -> bool> と From<T: AsRef<str>> の実装が衝突するので、後者は実装しない。
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

type BoxedReplacerFn<'cfg> =
    Box<dyn Fn(&[String], usize) -> anyhow::Result<(Vec<String>, usize)> + 'cfg>;

// From<F: Fn(...) -> anyhow::Result<...>> と From<T: AsRef<str>> の実装が衝突するので、後者は実装しない。
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
pub struct CommandLine<'cfg> {
    keywords: HashMap<String, ReplacerFn<'cfg>>,
    handlers: Vec<(PredicateFn<'cfg>, ReplacerFn<'cfg>)>,
}

impl<'cfg> CommandLine<'cfg> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_word_to_word(&mut self, word: &str, replaced: &'cfg str) -> bool {
        if !self.keywords.contains_key(word) {
            self.keywords.insert(word.to_string(), replaced.into());
            return true;
        }
        false
    }

    pub fn add_word_to_replacer<R>(&mut self, word: &str, replacer: R) -> bool
    where
        R: Fn(&[String], usize) -> anyhow::Result<(Vec<String>, usize)> + 'cfg,
    {
        if !self.keywords.contains_key(word) {
            self.keywords.insert(word.into(), replacer.into());
            return true;
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

    fn add_keyword(&mut self, keyword: &'cfg ZmKeyword) -> bool {
        match (keyword.prefix.as_ref(), keyword.mapping.as_ref()) {
            (Some(prefix), None) => {
                let pred = move |args: &[String], i: usize| {
                    args[i].starts_with(prefix) && args[i][prefix.len()..] == keyword.name
                };
                self.add_pred_to_word(pred, &keyword.name);
                true
            }
            (Some(prefix), Some(mapping)) => {
                mapping
                    .keys()
                    .map(|k| !self.keywords.contains_key(k) )
                    .all(|b| b)
                    .then(||
                        self.add_pred_to_replacer(
                            move |args, i| args[i].starts_with(prefix) && mapping.contains_key(&args[i][prefix.len()..]),
                            |args, i| Ok(( vec![mapping.get(&args[i][prefix.len()..]).unwrap().to_string()], i + 1))
                        )
                    )
                    .is_some()
            }
            (None, Some(mapping)) => {
                mapping
                    .keys()
                    .map(|k| !self.keywords.contains_key(k) )
                    .all(|b| b)
                    .then(||
                        self.add_pred_to_replacer(
                            move |args, i| mapping.contains_key(&args[i]),
                            |args, i| Ok((vec![mapping.get(&args[i]).unwrap().to_string()], i + 1)))
                        )
                    .is_some()
            }
            (None, None) => self.add_word_to_word(&keyword.name, &keyword.name),
        }
    }

    pub fn parse_args(&self, args: &[String]) -> anyhow::Result<Vec<String>> {
        let mut newer = vec![];

        let mut i = 0;
        while i < args.len() {
            let arg = &args[i];
            i = if let Some(replacer) = self.keywords.get(arg) {
                let (replaced, new_i) = replacer.0(args, i)?;
                newer.extend(replaced.to_vec());
                new_i
            } else if
                let ControlFlow::Break(res) = self.handlers.iter()
                    .try_for_each(|(pred, repl)|
                        pred.0(args, i)
                            .then(|| ControlFlow::Break(repl.0(args, i)))
                            .unwrap_or(ControlFlow::Continue(()))
                    )
            {
                let (replaced, new_i) = res?;
                newer.extend(replaced);
                new_i
            } else {
                newer.push(args[i].to_string());
                i + 1
            }
        }

        Ok(newer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handler_table_init_lifetime() {
        {
            // 先に HandlerTable のライフタイムが始まる場合
            let mut ht = CommandLine::new();
            ht.add_word_to_word("word", "replaced");

            let args = vec!["word".to_string()];

            let res = ht.parse_args(&args);
            assert!(res.is_ok());
            assert_eq!(res.unwrap(), &["replaced".to_string()]);
        }
        {
            // 先にコマンドライン引数のライフタイムが始まる場合
            let args = vec!["word".to_string()];

            let mut ht = CommandLine::new();
            ht.add_word_to_word("word", "replaced");

            let res = ht.parse_args(&args);
            assert!(res.is_ok());
            assert_eq!(res.unwrap(), &["replaced".to_string()]);
        }
    }
}
