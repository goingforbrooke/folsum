use iced::futures::channel::mpsc;
use iced::futures::sink::SinkExt;
use iced::futures::stream::StreamExt;
use iced::subscription::{self, Subscription};


// Define the kinds of processing events that can occur.
pub enum Event {
    Ready(mpsc::Sender<Input>),
    WorkFinished,
}

// Define the kinds of input events that can occur.
enum Input {
    DoSomeWork,
}

// Define the kinds of states that the worker thread can be in.
enum State {
    Starting,
    Ready(mpsc::Receiver<Input>),
}

// Define the task that the worker thread carries out.
fn some_worker() -> Subscription<Event> {
    struct SomeWorker;

    subscription::channel(std::any::TypeId::of::<SomeWorker>(), 100, |mut output| async move {
        let mut state = State::Starting;

        loop {
            match &mut state {
                State::Starting => {
                    // Create channel
                    let (sender, receiver) = mpsc::channel(100);

                    // Send the sender back to the application
                    output.send(Event::Ready(sender)).await;

                    // We are ready to receive messages
                    state = State::Ready(receiver);
                }
                State::Ready(receiver) => {
                    // Read next input sent from `Application`
                    let input = receiver.select_next_some().await;

                    match input {
                        Input::DoSomeWork => {
                            // Do some async work...

                            // Finally, we can optionally produce a message to tell the
                            // `Application` the work is done
                            output.send(Event::WorkFinished).await;
                        }
                    }
                }
            }
        }
    })
}










