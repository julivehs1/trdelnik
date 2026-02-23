//! Ring buffer implementations for O(1) indicator calculations

use std::collections::VecDeque;

/// A fixed-capacity ring buffer with O(1) average calculation.
///
/// This is used for SMA and Bollinger Bands calculations where we need
/// to maintain a running sum for efficient average computation.
#[derive(Debug, Clone)]
pub struct RingBuffer {
    buffer: Vec<f64>,
    capacity: usize,
    head: usize,
    len: usize,
    sum: f64,
    sum_sq: f64, // For standard deviation calculation
}

impl RingBuffer {
    /// Create a new ring buffer with the given capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: vec![0.0; capacity],
            capacity,
            head: 0,
            len: 0,
            sum: 0.0,
            sum_sq: 0.0,
        }
    }

    /// Push a new value into the buffer
    pub fn push(&mut self, value: f64) {
        if self.len < self.capacity {
            // Buffer not yet full
            self.buffer[self.len] = value;
            self.sum += value;
            self.sum_sq += value * value;
            self.len += 1;
        } else {
            // Buffer full - replace oldest value
            let old_value = self.buffer[self.head];
            self.sum -= old_value;
            self.sum_sq -= old_value * old_value;
            self.buffer[self.head] = value;
            self.sum += value;
            self.sum_sq += value * value;
            self.head = (self.head + 1) % self.capacity;
        }
    }

    /// Check if the buffer is full
    #[inline]
    pub fn is_full(&self) -> bool {
        self.len == self.capacity
    }

    /// Get the current length
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Check if the buffer is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Get the capacity
    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Get the running sum
    #[inline]
    pub fn sum(&self) -> f64 {
        self.sum
    }

    /// Calculate the average in O(1) time
    #[inline]
    pub fn average(&self) -> Option<f64> {
        if self.len == 0 {
            None
        } else {
            Some(self.sum / self.len as f64)
        }
    }

    /// Calculate the variance in O(1) time
    /// Uses the formula: Var(X) = E[X^2] - E[X]^2
    #[inline]
    pub fn variance(&self) -> Option<f64> {
        if self.len == 0 {
            None
        } else {
            let mean = self.sum / self.len as f64;
            let mean_sq = self.sum_sq / self.len as f64;
            Some(mean_sq - mean * mean)
        }
    }

    /// Calculate the standard deviation in O(1) time
    #[inline]
    pub fn std_dev(&self) -> Option<f64> {
        self.variance().map(|v| v.sqrt())
    }

    /// Get the most recently added value
    pub fn last(&self) -> Option<f64> {
        if self.len == 0 {
            None
        } else {
            let last_idx = if self.len < self.capacity {
                self.len - 1
            } else {
                (self.head + self.capacity - 1) % self.capacity
            };
            Some(self.buffer[last_idx])
        }
    }

    /// Reset the buffer
    pub fn reset(&mut self) {
        self.head = 0;
        self.len = 0;
        self.sum = 0.0;
        self.sum_sq = 0.0;
    }

    /// Iterate over values in order (oldest to newest)
    pub fn iter(&self) -> impl Iterator<Item = f64> + '_ {
        (0..self.len).map(move |i| {
            let idx = if self.len < self.capacity {
                i
            } else {
                (self.head + i) % self.capacity
            };
            self.buffer[idx]
        })
    }
}

/// A ring buffer that maintains O(1) min/max operations using a monotonic deque.
///
/// This is used for Stochastic Oscillator calculations where we need
/// efficient min/max over a sliding window.
#[derive(Debug, Clone)]
pub struct MinMaxRingBuffer {
    /// Values in insertion order
    values: VecDeque<f64>,
    /// Monotonically decreasing deque of (index, value) for max tracking
    max_deque: VecDeque<(usize, f64)>,
    /// Monotonically increasing deque of (index, value) for min tracking
    min_deque: VecDeque<(usize, f64)>,
    capacity: usize,
    /// Global index counter for tracking element age
    index: usize,
}

impl MinMaxRingBuffer {
    /// Create a new min/max ring buffer with the given capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            values: VecDeque::with_capacity(capacity),
            max_deque: VecDeque::with_capacity(capacity),
            min_deque: VecDeque::with_capacity(capacity),
            capacity,
            index: 0,
        }
    }

    /// Push a new value into the buffer
    pub fn push(&mut self, value: f64) {
        // Remove oldest value if buffer is full
        if self.values.len() == self.capacity {
            self.values.pop_front();
        }
        self.values.push_back(value);

        // Remove elements from the front that are now out of the window
        let min_valid_index = self.index.saturating_sub(self.capacity - 1);
        while let Some(&(idx, _)) = self.max_deque.front() {
            if idx < min_valid_index {
                self.max_deque.pop_front();
            } else {
                break;
            }
        }
        while let Some(&(idx, _)) = self.min_deque.front() {
            if idx < min_valid_index {
                self.min_deque.pop_front();
            } else {
                break;
            }
        }

        // For max: remove smaller elements from the back
        while let Some(&(_, v)) = self.max_deque.back() {
            if v <= value {
                self.max_deque.pop_back();
            } else {
                break;
            }
        }
        self.max_deque.push_back((self.index, value));

        // For min: remove larger elements from the back
        while let Some(&(_, v)) = self.min_deque.back() {
            if v >= value {
                self.min_deque.pop_back();
            } else {
                break;
            }
        }
        self.min_deque.push_back((self.index, value));

        self.index += 1;
    }

    /// Check if the buffer is full
    #[inline]
    pub fn is_full(&self) -> bool {
        self.values.len() == self.capacity
    }

    /// Get the current length
    #[inline]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Check if the buffer is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Get the capacity
    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Get the maximum value in O(1) time
    #[inline]
    pub fn max(&self) -> Option<f64> {
        self.max_deque.front().map(|&(_, v)| v)
    }

    /// Get the minimum value in O(1) time
    #[inline]
    pub fn min(&self) -> Option<f64> {
        self.min_deque.front().map(|&(_, v)| v)
    }

    /// Get the most recently added value
    #[inline]
    pub fn last(&self) -> Option<f64> {
        self.values.back().copied()
    }

    /// Reset the buffer
    pub fn reset(&mut self) {
        self.values.clear();
        self.max_deque.clear();
        self.min_deque.clear();
        self.index = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_buffer_basic() {
        let mut buf = RingBuffer::new(3);
        assert!(buf.is_empty());
        assert!(!buf.is_full());

        buf.push(1.0);
        buf.push(2.0);
        buf.push(3.0);

        assert!(buf.is_full());
        assert_eq!(buf.len(), 3);
        assert_eq!(buf.sum(), 6.0);
        assert_eq!(buf.average(), Some(2.0));
    }

    #[test]
    fn test_ring_buffer_overflow() {
        let mut buf = RingBuffer::new(3);
        buf.push(1.0);
        buf.push(2.0);
        buf.push(3.0);
        buf.push(4.0); // This should push out 1.0

        assert_eq!(buf.sum(), 9.0); // 2 + 3 + 4
        assert_eq!(buf.average(), Some(3.0));

        let values: Vec<f64> = buf.iter().collect();
        assert_eq!(values, vec![2.0, 3.0, 4.0]);
    }

    #[test]
    fn test_ring_buffer_std_dev() {
        let mut buf = RingBuffer::new(5);
        for v in [2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0] {
            buf.push(v);
        }
        // Last 5 values: 4.0, 5.0, 5.0, 7.0, 9.0
        // Mean = 6.0
        // Variance = ((4-6)^2 + (5-6)^2 + (5-6)^2 + (7-6)^2 + (9-6)^2) / 5 = (4+1+1+1+9)/5 = 3.2
        let variance = buf.variance().unwrap();
        assert!((variance - 3.2).abs() < 1e-10);
    }

    #[test]
    fn test_minmax_buffer_basic() {
        let mut buf = MinMaxRingBuffer::new(3);
        buf.push(5.0);
        buf.push(3.0);
        buf.push(7.0);

        assert_eq!(buf.min(), Some(3.0));
        assert_eq!(buf.max(), Some(7.0));
    }

    #[test]
    fn test_minmax_buffer_sliding() {
        let mut buf = MinMaxRingBuffer::new(3);
        buf.push(5.0);
        buf.push(3.0);
        buf.push(7.0);
        buf.push(2.0); // Now window is [3, 7, 2]

        assert_eq!(buf.min(), Some(2.0));
        assert_eq!(buf.max(), Some(7.0));

        buf.push(1.0); // Now window is [7, 2, 1]
        assert_eq!(buf.min(), Some(1.0));
        assert_eq!(buf.max(), Some(7.0));

        buf.push(0.5); // Now window is [2, 1, 0.5]
        assert_eq!(buf.min(), Some(0.5));
        assert_eq!(buf.max(), Some(2.0));
    }

    #[test]
    fn test_minmax_buffer_monotonic_increase() {
        let mut buf = MinMaxRingBuffer::new(3);
        buf.push(1.0);
        buf.push(2.0);
        buf.push(3.0);

        assert_eq!(buf.min(), Some(1.0));
        assert_eq!(buf.max(), Some(3.0));

        buf.push(4.0);
        assert_eq!(buf.min(), Some(2.0));
        assert_eq!(buf.max(), Some(4.0));
    }

    #[test]
    fn test_minmax_buffer_monotonic_decrease() {
        let mut buf = MinMaxRingBuffer::new(3);
        buf.push(5.0);
        buf.push(4.0);
        buf.push(3.0);

        assert_eq!(buf.min(), Some(3.0));
        assert_eq!(buf.max(), Some(5.0));

        buf.push(2.0);
        assert_eq!(buf.min(), Some(2.0));
        assert_eq!(buf.max(), Some(4.0));
    }
}
