use std::rc::Rc;
use std::borrow::Cow;
use rank::{Score, Ranker};

pub trait Fuzzable {
    fn pattern(&self) -> String;
}

impl Fuzzable for String {
    fn pattern(&self) -> String {
        self.clone()
    }
}

pub struct Fuzzer<T> {
    ranker: Ranker<T>,
    input: String,
    last_input: String,
    rankings: Option<Rc<Vec<(Score, usize)>>>,
}

impl<T> Fuzzer<T>
where
    T: Fuzzable,
{
    pub fn new(items: Vec<T>) -> Self {
        Self {
            ranker: Ranker::new(items),
            input: String::new(),
            last_input: String::new(),
            rankings: None,
        }
    }

    pub fn input(&self) -> &str {
        &self.input
    }

    pub fn input_mut(&mut self) -> &mut String {
        &mut self.input
    }

    pub fn matches(&mut self) -> Matches<T> {
        if self.last_input != self.input {
            self.last_input.clone_from(&self.input);
            self.rankings = Some(Rc::new(self.ranker.rankings_of(&self.input)))
        }

        Matches {
            rankings: self.get_rankings(),
            ranker: &self.ranker,
            idx: 0,
        }
    }

    pub fn get_rankings(&mut self) -> Rc<Vec<(Score, usize)>> {
        Rc::clone(self.rankings
            .get_or_insert_with(|| Rc::new(self.ranker.rankings_of(&self.input))))
    }
}

pub struct MatchOwned<T> {
    pub item: T,
    pub score: i64,
    pub indices: Vec<usize>,
}

impl<T> MatchOwned<T> {
    pub fn borrowed(&self) -> Match<T> {
        Match {
            item: &self.item,
            score: self.score,
            indices: Cow::Borrowed(&self.indices),
        }
    }
}

pub struct Match<'a, T> {
    pub item: &'a T,
    pub score: i64,
    pub indices: Cow<'a, Vec<usize>>,
}

impl<'a, T: Clone> Match<'a, T> {
    pub fn owned(&self) -> MatchOwned<T> {
        MatchOwned {
            item: self.item.clone(),
            score: self.score,
            indices: self.indices.to_vec(),
        }
    }
}

pub struct Matches<'a, T> {
    ranker: &'a Ranker<T>,
    rankings: Rc<Vec<(Score, usize)>>,
    idx: usize,
}

impl<'a, T> Iterator for Matches<'a, T>
where
    T: Fuzzable
{
    type Item = Match<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((score, item)) = self.rankings.get(self.idx).cloned() {
            self.idx += 1;
            self.ranker
                .choices()
                .get(item)
                .map(|item| Match {
                    item,
                    score: score.value,
                    indices: Cow::Owned(score.indices),
                })
        } else {
            None
        }
    }
}

mod rank;
