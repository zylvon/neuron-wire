//! Congestion control — AIMD (Additive Increase, Multiplicative Decrease).
//!
//! Provides a simple congestion controller for the UDP transport.
//! When ACKs arrive, the window grows additively; on loss, it shrinks
//! multiplicatively. This is a research-grade implementation — production
//! systems should use BBR or QUIC's congestion control.

/// AIMD congestion controller.
#[derive(Debug, Clone)]
pub struct AimdController {
    /// Current congestion window (in packets).
    pub cwnd: f64,
    /// Minimum window (floor).
    pub min_cwnd: f64,
    /// Maximum window (ceiling).
    pub max_cwnd: f64,
    /// Additive increase per ACK.
    pub ai: f64,
    /// Multiplicative decrease factor on loss.
    pub md: f64,
    /// Number of ACKs received (for AI timing).
    pub ack_count: u64,
    /// ACKs needed before one AI step.
    pub acks_per_increase: u64,
    /// Whether we're in slow start.
    pub slow_start: bool,
    /// Slow-start threshold.
    pub ssthresh: f64,
}

impl AimdController {
    /// Create a new AIMD controller with default parameters.
    pub fn new() -> Self {
        Self {
            cwnd: 2.0,
            min_cwnd: 1.0,
            max_cwnd: 64.0,
            ai: 1.0,
            md: 0.5,
            ack_count: 0,
            acks_per_increase: 1,
            slow_start: true,
            ssthresh: 16.0,
        }
    }

    /// Create with custom parameters.
    pub fn with_params(min: f64, max: f64, ai: f64, md: f64) -> Self {
        Self {
            cwnd: min,
            min_cwnd: min,
            max_cwnd: max,
            ai,
            md,
            ack_count: 0,
            acks_per_increase: 1,
            slow_start: true,
            ssthresh: max / 2.0,
        }
    }

    /// Called when an ACK is received. Returns the new window size.
    pub fn on_ack(&mut self) -> f64 {
        if self.slow_start {
            self.cwnd += 1.0;
            if self.cwnd >= self.ssthresh {
                self.slow_start = false;
            }
        } else {
            self.ack_count += 1;
            if self.ack_count >= self.acks_per_increase {
                self.cwnd += self.ai;
                self.ack_count = 0;
            }
        }
        self.cwnd = self.cwnd.min(self.max_cwnd);
        self.cwnd
    }

    /// Called when a loss is detected. Returns the new window size.
    pub fn on_loss(&mut self) -> f64 {
        self.ssthresh = (self.cwnd * self.md).max(self.min_cwnd);
        self.cwnd = self.ssthresh;
        self.slow_start = false;
        self.ack_count = 0;
        self.cwnd = self.cwnd.max(self.min_cwnd);
        self.cwnd
    }

    /// Check if we can send a packet given the current window.
    pub fn can_send(&self, in_flight: usize) -> bool {
        (in_flight as f64) < self.cwnd
    }

    /// Reset to initial state.
    pub fn reset(&mut self) {
        self.cwnd = self.min_cwnd;
        self.ack_count = 0;
        self.slow_start = true;
        self.ssthresh = self.max_cwnd / 2.0;
    }

    /// Current window as an integer (packet count).
    pub fn window_packets(&self) -> usize {
        self.cwnd.ceil() as usize
    }
}

impl Default for AimdController {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let c = AimdController::new();
        assert_eq!(c.cwnd, 2.0);
        assert!(c.slow_start);
    }

    #[test]
    fn test_slow_start_growth() {
        let mut c = AimdController::new();
        let w0 = c.cwnd;
        c.on_ack();
        assert!(c.cwnd > w0, "window should grow in slow start");
    }

    #[test]
    fn test_loss_reduces_window() {
        let mut c = AimdController::new();
        for _ in 0..10 {
            c.on_ack();
        }
        let before = c.cwnd;
        c.on_loss();
        assert!(c.cwnd < before, "loss should reduce window");
        assert!(c.cwnd >= c.min_cwnd, "window should not go below min");
    }

    #[test]
    fn test_can_send() {
        let c = AimdController::new();
        assert!(c.can_send(0));
        assert!(c.can_send(1));
        assert!(!c.can_send(100));
    }

    #[test]
    fn test_reset() {
        let mut c = AimdController::new();
        for _ in 0..10 {
            c.on_ack();
        }
        c.on_loss();
        c.reset();
        assert_eq!(c.cwnd, c.min_cwnd);
        assert!(c.slow_start);
    }

    #[test]
    fn test_window_packets() {
        let c = AimdController::new();
        assert!(c.window_packets() >= 1);
    }

    #[test]
    fn test_custom_params() {
        let c = AimdController::with_params(1.0, 32.0, 1.0, 0.5);
        assert_eq!(c.max_cwnd, 32.0);
        assert_eq!(c.min_cwnd, 1.0);
    }
}
