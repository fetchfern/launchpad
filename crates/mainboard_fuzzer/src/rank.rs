use std::cmp::Reverse;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

#[derive(Default, Debug, Clone)]
pub struct Score {
    pub value: i64,
    pub indices: Vec<usize>,
}

impl Score {
    pub const fn new(value: i64, indices: Vec<usize>) -> Self {
        Score { value, indices } 
    }
}

pub struct Ranker<T> {
    matcher: SkimMatcherV2,
    choices: Vec<T>,
}

impl<T> Ranker<T>
where
    T: super::Fuzzable,
{
    pub fn new(choices: Vec<T>) -> Self {
        Self {
            matcher: SkimMatcherV2::default(),
            choices,
        }
    }

    pub fn choices(&self) -> &[T] {
        &self.choices
    }

    pub fn rankings_of(&mut self, input: &str) -> Vec<(Score, usize)> {
        let mut scores = Vec::with_capacity(self.choices.len());

        for (index, choice) in self.choices.iter().enumerate() {
            let score = self.matcher
                .fuzzy_indices(&choice.pattern(), input)
                .map(|(score, indices)| Score::new(score, indices))
                .unwrap_or_default();

            scores.push((score, index));
        }

        scores.sort_by_key(|v| Reverse(v.0.value));

        scores
    }
}

