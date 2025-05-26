use tokio::sync::mpsc;
use tokio::time::{Duration, timeout};
use zeromq::ZmqMessage;

#[tokio::test]
async fn test_iopub_channel_backpressure() {
    // Create a bounded channel with small capacity
    let (tx, mut rx) = mpsc::channel::<ZmqMessage>(2);

    // Fill the channel
    tx.send(ZmqMessage::from(vec![0u8])).await.unwrap();
    tx.send(ZmqMessage::from(vec![1u8])).await.unwrap();

    // Attempt to send a third message: should block (timeout)
    let send_third = tx.send(ZmqMessage::from(vec![2u8]));
    let res = timeout(Duration::from_millis(100), send_third).await;
    assert!(res.is_err(), "Expected send to block when channel is full");

    // Consume one message to free capacity
    let _ = rx.recv().await;

    // Now sending should complete without blocking
    let send_fourth = tx.send(ZmqMessage::from(vec![3u8]));
    let res2 = timeout(Duration::from_millis(100), send_fourth).await;
    assert!(res2.is_ok(), "Expected send to succeed after space freed");
}
