extern crate time;

use ::CONFIG;
use self::time::PreciseTime;
use serenity::client::Context;
use serenity::model::Message;

pub fn handler(context: &Context, message: &Message, _args: Vec<String>) -> Result<(), String> {
    if !owner_check(context, message) {
        return Ok(());
    }

    let start = PreciseTime::now();
    let msg = context.say("0");
    let end = PreciseTime::now();
    if let Ok(mut m) = msg {
        let ms = start.to(end).num_milliseconds();
        let _ = m.edit(&format!("Pong, {} milliseconds", ms), |m| m);
    }

    Ok(())
}

fn owner_check(_: &Context, message: &Message) -> bool {
    CONFIG.owners.contains(&message.author.id.0)
}
