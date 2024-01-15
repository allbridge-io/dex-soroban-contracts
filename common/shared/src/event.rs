use soroban_sdk::{Env, IntoVal, Symbol, Val};

pub trait Event: IntoVal<Env, Val> + Sized {
    const EVENT_NAME: &'static str;

    fn publish(self, env: &Env) {
        env.events()
            .publish((Symbol::new(env, Self::EVENT_NAME),), self);
    }
}
