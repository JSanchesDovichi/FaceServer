use std::time::{Duration, SystemTime};
use tokio_cron_scheduler::{Job, JobScheduler, JobSchedulerError};
use rocket::{get, launch, routes};

#[launch]
async fn rocket() -> _ {
    let mut sched = JobScheduler::new().await.unwrap();

    sched.add(
        Job::new("1/2 * * * * *", |_uuid, _l| {
            println!("I run every 2 seconds [{:?}]", SystemTime::now());
        }).unwrap()
    ).await.unwrap();

    // Start the scheduler
    sched.start().await.unwrap();

    rocket::build().mount("/", routes![hello])
}

#[get("/<name>/<age>")]
fn hello(name: &str, age: u8) -> String {
    format!("Hello, {} year old named {}!", age, name)
}