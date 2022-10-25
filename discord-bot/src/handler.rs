use serenity::{
    async_trait,
    model::prelude::Ready,
    prelude::{Context, EventHandler},
};

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}
