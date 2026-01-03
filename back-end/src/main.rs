use rocket::serde::json::Value;
use std::fs;
use std::time::{Duration, SystemTime};
use tokio_cron_scheduler::{Job, JobScheduler, JobSchedulerError};
use rocket::{get, post, launch, routes, FromForm, State};
use rocket_dyn_templates::{Template, context};
use rocket::form::Form;
use rocket::serde::{Deserialize, Serialize, json::Json};
use rocket::fs::TempFile;
use rocket::http::ext::IntoCollection;
use surrealdb::{Error, Surreal};
use surrealdb::opt::auth::Root;
//use surrealdb::engine::local::RocksDb;
use surrealdb::engine::local::{Db, RocksDb};
use surrealdb::engine::remote::ws::Client;
use surrealdb::method::Select;
use surrealdb::types::{SurrealValue};

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
async fn upload_form(mut upload: Form<Upload<'_>>, db: &State<Surreal<Db>>) {
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

#[get("/test_db")]
async fn test_db(db: &State<Surreal<Db>>) {
    #[derive(Serialize, Deserialize, Debug, SurrealValue)]
    #[serde(crate = "rocket::serde")]
    struct Person {
        name: String,
    }

    /*
    match db.query("CREATE person:tobie SET name = 'Tobie'").await {
        Ok(rows) => {
            println!("{:#?}", rows);
        },
        Err(e) => {}
    }

    match db.query("SELECT * FROM person").await {
        Ok(rows) => {
            println!("{:#?}", rows);
        },
        Err(e) => {}
    }
     */

    // Create a new person with a random ID
    let created: Option<Person> = db.create(("person", "tobie"))
        .content(Person {
            name: "Tobie".to_string()
        })
        .await.unwrap();

    // Select all people records
    let people: Vec<Person> = db.select("person").await.unwrap();

    println!("{:?}", people);
}

#[launch]
async fn rocket() -> _ {
    if let Err(e) = setup_scheduled_task().await {
        panic!("Failed to setup scheduled task: {:?}", e);
    }

    let db = match Surreal::new::<RocksDb>("./target/debug/database").await {
        Ok(db) => {
            // Select a specific namespace / database
            if let Err(e) = db.use_ns("namespace").use_db("database").await {
                panic!("Failed to determine namespace and database {:?}", e);
            }

            db
        },
        Err(e) => panic!("Failed to setup database: {:?}", e)
    };

    rocket::build()
        .mount("/", routes![index, upload_form, test_db])
        .manage(db)
        .attach(Template::fairing())
}
