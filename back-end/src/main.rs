use std::fs;
use std::time::{Duration, SystemTime};
use tokio_cron_scheduler::{Job, JobScheduler, JobSchedulerError};
use rocket::{get, post, launch, routes, FromForm};
use rocket_dyn_templates::{Template, context};
use rocket::form::Form;
use rocket::fs::TempFile;

async fn setup_scheduled_task() -> Result<(), JobSchedulerError> {
    let mut sched = JobScheduler::new().await?;

    sched.add(
        Job::new("1/2 * * * * *", |_uuid, _l| {
            //println!("I run every 2 seconds [{:?}]", SystemTime::now());
            let paths = fs::read_dir("./downloads").unwrap();

            println!("There are {} photos waiting for training!", paths.count());
        })?
    ).await?;

    // Start the scheduler
    sched.start().await
}

#[get("/")]
fn index() -> Template {
    Template::render("index", context! { field: "value" })
}

#[derive(FromForm)]
struct Upload<'r> {
    //save: bool,
    files: Vec<TempFile<'r>>,
}

#[post("/upload_image", data = "<upload>")]
async fn upload_form(mut upload: Form<Upload<'_>>) {
    for file in &mut upload.files {
        println!("Received to {:?}", file.path());

        let final_path = format!("./downloads/{}", file.name().unwrap());

        match file.copy_to(final_path).await {
            Ok(_) => {},
            Err(e) => {
                println!("Failed to persist {:?}", e);
            }
        }
    }
}

#[launch]
async fn rocket() -> _ {
    if let Err(e) = setup_scheduled_task().await {
        panic!("Failed to setup scheduled task: {:?}", e);
    }

    rocket::build().mount("/", routes![index, upload_form]).attach(Template::fairing())
}
