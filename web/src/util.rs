use dioxus::{core::ReactiveContext, prelude::*};
use futures_util::{FutureExt, StreamExt, pin_mut};
use std::{cell::Cell, future, rc::Rc};

#[doc(alias = "use_async_memo")]
#[doc(alias = "use_memo_async")]
#[must_use = "Consider using `cx.spawn` to run a future without reading its value"]
#[track_caller]
pub fn use_store_resource<T, F>(mut future: impl FnMut() -> F + 'static) -> Store<Option<T>>
where
    T: 'static,
    F: Future<Output = T> + 'static,
{
    let location = std::panic::Location::caller();

    let mut value = use_store(|| None);
    let (rc, changed) = use_hook(|| {
        let (rc, changed) = ReactiveContext::new_with_origin(location);
        (rc, Rc::new(Cell::new(Some(changed))))
    });

    let cb = use_callback(move |_| {
        // Create the user's task
        let fut = rc.reset_and_run_in(&mut future);

        // Spawn a wrapper task that polls the inner future and watch its dependencies
        spawn(async move {
            // move the future here and pin it so we can poll it
            let fut = fut;
            pin_mut!(fut);

            // Run each poll in the context of the reactive scope
            // This ensures the scope is properly subscribed to the future's dependencies
            let res = future::poll_fn(|cx| rc.run_in(|| fut.poll_unpin(cx))).await;

            // Set the value
            value.set(Some(res));
        })
    });

    let mut task = use_hook(|| Signal::new(cb(())));

    use_hook(|| {
        let mut changed = changed.take().unwrap();
        spawn(async move {
            loop {
                // Wait for the dependencies to change
                let _ = changed.next().await;

                // Stop the old task
                task.write().cancel();

                // Start a new task
                task.set(cb(()));
            }
        })
    });

    value
}
