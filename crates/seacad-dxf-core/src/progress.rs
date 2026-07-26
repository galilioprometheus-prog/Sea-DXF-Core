use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

/// Monotonic byte progress for one bounded source operation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfReadProgress {
    processed_bytes: u64,
    total_bytes: u64,
}

impl DxfReadProgress {
    /// Returns None when progress would exceed the known source length.
    #[must_use]
    pub const fn new(processed_bytes: u64, total_bytes: u64) -> Option<Self> {
        if processed_bytes <= total_bytes {
            Some(Self {
                processed_bytes,
                total_bytes,
            })
        } else {
            None
        }
    }

    #[must_use]
    pub const fn processed_bytes(self) -> u64 {
        self.processed_bytes
    }

    #[must_use]
    pub const fn total_bytes(self) -> u64 {
        self.total_bytes
    }

    #[must_use]
    pub const fn is_complete(self) -> bool {
        self.processed_bytes == self.total_bytes
    }
}

/// Return value from a synchronous read progress callback.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum DxfReadControl {
    #[default]
    Continue,
    Cancel,
}

/// Receives bounded progress and may request cooperative cancellation.
pub trait DxfReadObserver {
    fn on_progress(&mut self, progress: DxfReadProgress) -> DxfReadControl;
}

impl<F> DxfReadObserver for F
where
    F: FnMut(DxfReadProgress) -> DxfReadControl,
{
    fn on_progress(&mut self, progress: DxfReadProgress) -> DxfReadControl {
        self(progress)
    }
}

/// Observer for callers that do not need progress events.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoopDxfReadObserver;

impl DxfReadObserver for NoopDxfReadObserver {
    fn on_progress(&mut self, _progress: DxfReadProgress) -> DxfReadControl {
        DxfReadControl::Continue
    }
}

/// Cloneable, thread-safe cancellation signal for a synchronous operation.
#[derive(Clone, Debug, Default)]
pub struct DxfCancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl DxfCancellationToken {
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }

    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DxfCancellationToken, DxfReadControl, DxfReadObserver, DxfReadProgress, NoopDxfReadObserver,
    };

    fn assert_send_sync<T: Send + Sync>() {}

    #[test]
    fn progress_rejects_values_beyond_the_source() {
        let progress = DxfReadProgress::new(5, 8);
        assert_eq!(progress.map(DxfReadProgress::processed_bytes), Some(5));
        assert_eq!(progress.map(DxfReadProgress::total_bytes), Some(8));
        assert_eq!(progress.map(DxfReadProgress::is_complete), Some(false));
        assert_eq!(DxfReadProgress::new(9, 8), None);
        assert_eq!(
            DxfReadProgress::new(8, 8).map(DxfReadProgress::is_complete),
            Some(true)
        );
    }

    #[test]
    fn closures_and_noop_observer_follow_the_callback_contract() {
        let mut observed = 0;
        let mut observer = |progress: DxfReadProgress| {
            observed = progress.processed_bytes();
            DxfReadControl::Cancel
        };
        let progress = DxfReadProgress::new(4, 10);
        if let Some(progress) = progress {
            assert_eq!(observer.on_progress(progress), DxfReadControl::Cancel);
        }
        assert_eq!(observed, 4);

        let mut noop = NoopDxfReadObserver;
        if let Some(progress) = progress {
            assert_eq!(noop.on_progress(progress), DxfReadControl::Continue);
        }
    }

    #[test]
    fn cancellation_is_shared_and_thread_safe() {
        assert_send_sync::<DxfCancellationToken>();
        let token = DxfCancellationToken::default();
        let second_view = token.clone();
        assert!(!second_view.is_cancelled());
        token.cancel();
        assert!(second_view.is_cancelled());
    }
}
