mod elastic;
mod telegram_bot;
mod types;

#[tokio::main]
async fn main() {
    pretty_env_logger::init_timed();
    // let rocket_future = rocket::build().mount("/", routes![]).launch();

    let teloxide_future = telegram_bot::start();

    let _ = futures::join!(teloxide_future);
}
