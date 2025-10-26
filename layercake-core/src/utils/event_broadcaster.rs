use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

/// Generic event broadcaster for pub/sub patterns with automatic channel management.
///
/// This broadcaster provides:
/// - Type-safe event distribution to multiple subscribers
/// - Automatic channel creation on first subscription
/// - Memory cleanup for idle channels (no active receivers)
/// - Thread-safe access with optimized read/write lock patterns
///
/// # Type Parameters
///
/// * `K` - Key type for identifying channels (must be Eq + Hash + Clone)
/// * `V` - Event value type (must be Clone for broadcast)
///
/// # Examples
///
/// ```rust,no_run
/// use layercake::utils::EventBroadcaster;
///
/// # async fn example() {
/// // Create a broadcaster for string events
/// let broadcaster = EventBroadcaster::<String, String>::new(1000);
///
/// // Subscribe to a plan's events
/// let mut receiver = broadcaster.subscribe("plan-123".to_string()).await;
///
/// // Publish an event
/// broadcaster.publish("plan-123".to_string(), "event data".to_string()).await.ok();
///
/// // Clean up idle channels periodically
/// broadcaster.cleanup_idle().await;
/// # }
/// ```
pub struct EventBroadcaster<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    channels: Arc<RwLock<HashMap<K, broadcast::Sender<V>>>>,
    buffer_size: usize,
}

impl<K, V> EventBroadcaster<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    /// Create a new event broadcaster with the specified buffer size.
    ///
    /// The buffer size determines how many events can be queued per channel
    /// before old events are dropped for slow subscribers.
    ///
    /// # Arguments
    ///
    /// * `buffer_size` - Number of events to buffer per channel
    pub fn new(buffer_size: usize) -> Self {
        Self {
            channels: Arc::new(RwLock::new(HashMap::new())),
            buffer_size,
        }
    }

    /// Subscribe to events for a specific key.
    ///
    /// Creates a new channel if one doesn't exist. Multiple subscribers
    /// can listen to the same key.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to subscribe to
    ///
    /// # Returns
    ///
    /// A receiver that will receive all events published to this key
    pub async fn subscribe(&self, key: K) -> broadcast::Receiver<V> {
        self.get_or_create(key).await.subscribe()
    }

    /// Publish an event to all subscribers of a key.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to publish to
    /// * `event` - The event to broadcast
    ///
    /// # Returns
    ///
    /// * `Ok(usize)` - Number of receivers that received the event (0 if no active receivers)
    ///
    /// # Note
    ///
    /// If there are no active receivers, returns Ok(0) and the event is dropped.
    /// Call `cleanup_idle()` periodically to remove unused channels.
    pub async fn publish(&self, key: K, event: V) -> Result<usize, String> {
        let sender = self.get_or_create(key).await;
        // broadcast::send returns Err when there are no receivers,
        // but we want to return Ok(0) in that case since the channel exists
        match sender.send(event) {
            Ok(count) => Ok(count),
            Err(_) => Ok(0), // No receivers, but not an error condition
        }
    }

    /// Get the number of active receivers for a specific key.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to check
    ///
    /// # Returns
    ///
    /// The number of active receivers, or 0 if the channel doesn't exist
    pub async fn receiver_count(&self, key: &K) -> usize {
        let channels = self.channels.read().await;
        channels
            .get(key)
            .map(|sender| sender.receiver_count())
            .unwrap_or(0)
    }

    /// Remove all channels with no active receivers.
    ///
    /// This should be called periodically (e.g., every minute) to prevent
    /// memory leaks from unused channels accumulating.
    ///
    /// # Returns
    ///
    /// The number of channels that were cleaned up
    pub async fn cleanup_idle(&self) -> usize {
        let mut channels = self.channels.write().await;
        let before = channels.len();
        channels.retain(|_, sender| sender.receiver_count() > 0);
        let after = channels.len();
        before - after
    }

    /// Get the total number of channels (both active and idle).
    pub async fn channel_count(&self) -> usize {
        let channels = self.channels.read().await;
        channels.len()
    }

    /// Get or create a broadcast sender for a specific key.
    ///
    /// Uses the double-checked locking pattern for optimal performance:
    /// 1. Try read lock first (fast path) - most requests hit this
    /// 2. If not found, acquire write lock (slow path) and create channel
    /// 3. Double-check after acquiring write lock to avoid race conditions
    async fn get_or_create(&self, key: K) -> broadcast::Sender<V> {
        // Fast path: Try read lock first and immediately release
        {
            let channels = self.channels.read().await;
            if let Some(sender) = channels.get(&key) {
                return sender.clone();
            }
            // Lock automatically dropped here
        }

        // Slow path: Need to create channel with write lock
        let mut channels = self.channels.write().await;

        // Double-check pattern to avoid race conditions
        // (another thread might have created it while we waited for write lock)
        if let Some(sender) = channels.get(&key) {
            sender.clone()
        } else {
            let (sender, _) = broadcast::channel(self.buffer_size);
            channels.insert(key, sender.clone());
            sender
        }
    }
}

// Implement Clone for EventBroadcaster so it can be shared across async tasks
impl<K, V> Clone for EventBroadcaster<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    fn clone(&self) -> Self {
        Self {
            channels: Arc::clone(&self.channels),
            buffer_size: self.buffer_size,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_subscribe_and_publish() {
        let broadcaster = EventBroadcaster::<i32, String>::new(10);

        let mut receiver = broadcaster.subscribe(1).await;

        let result = broadcaster.publish(1, "test event".to_string()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1); // One receiver

        let received = receiver.recv().await.unwrap();
        assert_eq!(received, "test event");
    }

    #[tokio::test]
    async fn test_multiple_subscribers() {
        let broadcaster = EventBroadcaster::<String, i32>::new(10);

        let mut receiver1 = broadcaster.subscribe("key1".to_string()).await;
        let mut receiver2 = broadcaster.subscribe("key1".to_string()).await;

        let result = broadcaster.publish("key1".to_string(), 42).await;
        assert_eq!(result.unwrap(), 2); // Two receivers

        assert_eq!(receiver1.recv().await.unwrap(), 42);
        assert_eq!(receiver2.recv().await.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_cleanup_idle() {
        let broadcaster = EventBroadcaster::<i32, String>::new(10);

        // Create a channel and drop the receiver
        {
            let _receiver = broadcaster.subscribe(1).await;
            assert_eq!(broadcaster.receiver_count(&1).await, 1);
        }

        // Receiver dropped, channel should be idle
        assert_eq!(broadcaster.receiver_count(&1).await, 0);

        // Cleanup should remove it
        let cleaned = broadcaster.cleanup_idle().await;
        assert_eq!(cleaned, 1);
        assert_eq!(broadcaster.channel_count().await, 0);
    }

    #[tokio::test]
    async fn test_no_receivers_error() {
        let broadcaster = EventBroadcaster::<i32, String>::new(10);

        // Create and immediately drop receiver
        {
            let _receiver = broadcaster.subscribe(1).await;
        }

        // Publishing with no receivers should still succeed (channel exists)
        // But returns 0 receivers
        let result = broadcaster.publish(1, "test".to_string()).await;
        assert_eq!(result.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        let broadcaster = Arc::new(EventBroadcaster::<i32, i32>::new(100));

        // Spawn multiple tasks subscribing and publishing
        let mut handles = vec![];

        for i in 0..10 {
            let bc = Arc::clone(&broadcaster);
            handles.push(tokio::spawn(async move {
                let mut receiver = bc.subscribe(i % 3).await;
                bc.publish(i % 3, i).await.ok();
                receiver.recv().await.ok()
            }));
        }

        // All should complete without panicking
        for handle in handles {
            handle.await.ok();
        }
    }
}
