use soroban_sdk::{Env, IntoVal, Symbol, Val};

/// # Example
/// ```
/// use shared::Event;
/// use soroban_sdk::{contracttype, BytesN};
///
/// #[contracttype]
/// pub struct MessageReceived {
///     pub message: BytesN<32>,
/// }
///
/// impl Event for MessageReceived {
///     const EVENT_NAME: &'static str = "MessageReceived";
/// }
/// ```
pub trait Event: IntoVal<Env, Val> + Sized {
    const EVENT_NAME: &'static str;

    fn publish(self, env: &Env) {
        env.events()
            .publish((Symbol::new(env, Self::EVENT_NAME),), self);
    }
}
