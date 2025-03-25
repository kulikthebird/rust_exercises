use std::cmp::Ordering;
use std::collections::{BTreeMap, VecDeque};
use std::iter::Extend;

use serde::Serialize;

const MAX_WINDOW_SIZE: usize = 100000000;
const MAX_INPUT_SIZE: usize = 10000;

#[derive(Debug, PartialEq, PartialOrd, Clone)]
struct FloatOrd(f64);

impl Eq for FloatOrd {}

#[allow(clippy::derive_ord_xor_partial_ord)]
impl Ord for FloatOrd {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.total_cmp(&other.0) // Use total_cmp() for correct ordering
    }
}

/// The structure keeps the pre-calculated stats of a given symbol.
///
/// TODO: We use a binary tree to get min/max values in O(k*log(n)),
/// where k is the number of new data points in a batch and n is a window size.
/// * In theory it sounds nice, but using Btree is not cache friendly.
/// * BinaryHeap does not support a O(log(n)) time item removal, so we can't use it.
/// * It may happen, that using a simple min/max function on an array may give much
///   better performance in practice.
#[derive(Clone, Debug)]
pub(crate) struct Stats {
    stats_resp: StatsResponse,
    partial_sum: f64,
    partial_squared_sum: f64,
    ordered_values: BTreeMap<FloatOrd, usize>,
    window_size: usize,
}

impl Stats {
    pub fn new(window_size: usize) -> Self {
        Stats {
            stats_resp: StatsResponse::default(),
            partial_sum: 0.,
            partial_squared_sum: 0.,
            ordered_values: BTreeMap::new(),
            window_size,
        }
    }

    pub fn get_stats_resp(&self) -> &StatsResponse {
        &self.stats_resp
    }

    /// Updates the stats
    ///
    /// Time complexities:
    ///  * min/max - O(k*log(n))
    ///  * last - O(1)
    ///  * avg, var - O(k)
    ///
    ///   where k is the number of data points from the latest batch
    ///   and n is a window size.
    ///
    pub fn update_stats(&mut self, num_of_new_values: usize, values: &VecDeque<f64>) {
        let current_window_size = usize::min(self.window_size, values.len());

        // Iterator over the recently added data points
        let values_added_to_window = &values.range(
            values
                .len()
                .saturating_sub(usize::min(current_window_size, num_of_new_values))..,
        );

        // Iterator over values that were moved out
        // of the current window by the recent data insertion.
        let old_values_index = values.len() - num_of_new_values;
        let values_removed_from_window = &values.range(
            old_values_index.saturating_sub(self.window_size)
                ..usize::min(
                    (old_values_index + num_of_new_values).saturating_sub(self.window_size),
                    old_values_index,
                ),
        );

        // Recalculate stats
        self.partial_sum += values_added_to_window.clone().sum::<f64>()
            - values_removed_from_window.clone().sum::<f64>();
        self.partial_squared_sum += values_added_to_window
            .clone()
            .fold(0., |acc, a| acc + a * a)
            - values_removed_from_window
                .clone()
                .fold(0., |acc, a| acc + a * a);

        // Remove all unused items from the binary tree
        values_removed_from_window.clone().for_each(|x| {
            let entry_count = self
                .ordered_values
                .remove(&FloatOrd(*x))
                .map(|c| c - 1)
                .expect("The value should be present");
            if entry_count != 0 {
                self.ordered_values.insert(FloatOrd(*x), entry_count);
            }
        });

        // Add new items to the binary tree
        values_added_to_window.clone().for_each(|x| {
            self.ordered_values
                .entry(FloatOrd(*x))
                .and_modify(|x| *x += 1)
                .or_insert(1);
        });

        self.stats_resp = StatsResponse {
            min: self
                .ordered_values
                .first_key_value()
                .map(|(x, _)| x.0)
                .unwrap_or(f64::NEG_INFINITY),
            max: self
                .ordered_values
                .last_key_value()
                .map(|(x, _)| x.0)
                .unwrap_or(f64::INFINITY),
            last: values.iter().last().unwrap_or(&f64::NAN).to_owned(),
            avg: self.partial_sum / f64::from(current_window_size as u32),
            var: self.partial_squared_sum / f64::from(current_window_size as u32)
                - (self.partial_sum / f64::from(current_window_size as u32)).powi(2),
        };
    }
}

#[derive(Clone)]
pub(crate) struct Symbol {
    /// We use VecDeque which is essentially a ring buffer.
    /// It should prevent any reallocations and reduce the time complexity.
    /// TODO: make sure that it does not reallocate the buffer.
    items: VecDeque<f64>,
    stats: [Stats; 8],
}

impl Default for Symbol {
    fn default() -> Self {
        Self {
            items: VecDeque::with_capacity(MAX_WINDOW_SIZE + MAX_INPUT_SIZE),
            stats: [
                Stats::new(10),
                Stats::new(100),
                Stats::new(1000),
                Stats::new(10000),
                Stats::new(100000),
                Stats::new(1000000),
                Stats::new(10000000),
                Stats::new(100000000),
            ],
        }
    }
}

impl Symbol {
    pub fn get_stats(&self, stats_index: usize) -> Option<&Stats> {
        self.stats.get(stats_index)
    }

    pub fn insert(&mut self, values: &[f64]) {
        self.items.extend(values);

        // Update stats
        for stats in self.stats.iter_mut() {
            stats.update_stats(values.len(), &self.items);
        }

        // Remove the values beyond the max window size - we won't need it anymore.
        if self.items.len() > MAX_WINDOW_SIZE {
            self.items.drain(0..self.items.len() - MAX_WINDOW_SIZE);
        }
    }
}

#[derive(Serialize, Clone, Default, Copy, Debug, PartialEq)]
pub(crate) struct StatsResponse {
    min: f64,
    max: f64,
    last: f64,
    avg: f64,
    var: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        let symbol = Symbol::default();
        assert_eq!(
            symbol.get_stats(0).unwrap().get_stats_resp(),
            &StatsResponse {
                min: 0.,
                max: 0.,
                last: 0.,
                avg: 0.,
                var: 0.
            },
        );
        assert_eq!(
            symbol.get_stats(7).unwrap().get_stats_resp(),
            &StatsResponse {
                min: 0.,
                max: 0.,
                last: 0.,
                avg: 0.,
                var: 0.
            },
        );
    }

    #[test]
    fn test_single_entry() {
        let mut symbol = Symbol::default();
        symbol.insert(&[1., 2., 3.]);
        assert_eq!(
            symbol.get_stats(0).unwrap().get_stats_resp(),
            &StatsResponse {
                min: 1.0,
                max: 3.0,
                last: 3.0,
                avg: 2.,
                var: 0.666666666666667
            },
        );
        assert_eq!(
            symbol.get_stats(7).unwrap().get_stats_resp(),
            &StatsResponse {
                min: 1.0,
                max: 3.0,
                last: 3.0,
                avg: 2.,
                var: 0.666666666666667
            },
        );
    }

    #[test]
    fn test_all_window_sizes() {
        let input: Vec<f64> = (0..10000).map(f64::from).collect();

        let mut symbol = Symbol::default();
        symbol.insert(&input);
        symbol.insert(&input);
        assert_eq!(
            symbol.get_stats(0).unwrap().get_stats_resp(),
            &StatsResponse {
                min: 9990.,
                max: 9999.,
                last: 9999.0,
                avg: 9994.5,
                var: 8.25
            },
        );
        assert_eq!(
            symbol.get_stats(1).unwrap().get_stats_resp(),
            &StatsResponse {
                min: 9900.,
                max: 9999.,
                last: 9999.0,
                avg: 9949.5,
                var: 833.25
            },
        );
        assert_eq!(
            symbol.get_stats(2).unwrap().get_stats_resp(),
            &StatsResponse {
                min: 9000.,
                max: 9999.,
                last: 9999.0,
                avg: 9499.5,
                var: 83333.25
            },
        );
        assert_eq!(
            symbol.get_stats(3).unwrap().get_stats_resp(),
            &StatsResponse {
                min: 0.,
                max: 9999.,
                last: 9999.0,
                avg: 4999.5,
                var: 8333333.25
            }
        );
        assert_eq!(
            symbol.get_stats(4).unwrap().get_stats_resp(),
            &StatsResponse {
                min: 0.,
                max: 9999.,
                last: 9999.0,
                avg: 4999.5,
                var: 8333333.25
            },
        );
        assert_eq!(
            symbol.get_stats(5).unwrap().get_stats_resp(),
            &StatsResponse {
                min: 0.,
                max: 9999.,
                last: 9999.0,
                avg: 4999.5,
                var: 8333333.25
            },
        );
        assert_eq!(
            symbol.get_stats(6).unwrap().get_stats_resp(),
            &StatsResponse {
                min: 0.,
                max: 9999.,
                last: 9999.0,
                avg: 4999.5,
                var: 8333333.25
            },
        );
        assert_eq!(
            symbol.get_stats(7).unwrap().get_stats_resp(),
            &StatsResponse {
                min: 0.,
                max: 9999.,
                last: 9999.0,
                avg: 4999.5,
                var: 8333333.25
            },
        );
    }
}
