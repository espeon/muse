pub mod db;

use futures::{
    channel::mpsc::{channel, Receiver},
    SinkExt, StreamExt,
};

use sqlx::postgres::Postgres;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use notify::event::EventKind;
use std::{path::Path, thread, time::{self, Duration}};
use jwalk::WalkDir;

use crate::metadata;

pub async fn start<P: AsRef<Path>>(path: P, pathstr:&str, pool:sqlx::Pool<Postgres>) {
    if Path::new(&pathstr).is_file(){
        return println!("critical error !!!!!\nthe path {} is not a folder.",&pathstr)
    }
    println!("scanning folder {:?}", &pathstr);
    scan(&path, pool.clone()).await;
    println!("watching folder {:?}", pathstr);
    if let Err(e) = watch(path, pool).await {
        println!("error: {:?}", e)
    }
}

pub async fn scan<P: AsRef<Path>>(path: P, pool:sqlx::Pool<Postgres>) {
    // wait for other stuff to finish logging
    // will remove this sooner or later
    thread::sleep(time::Duration::from_millis(250));
    for entry in WalkDir::new(path).sort(true) {
        let ent = &entry.unwrap();
        if ent.path().is_file() {
            metadata::scan_file(&ent.path(), pool.clone()).await;
        }
      }
}

fn async_watcher() -> notify::Result<(RecommendedWatcher, Receiver<notify::Result<Event>>)> {
    let (mut tx, rx) = channel(1);

    let config = notify::Config::default()
    .with_poll_interval(Duration::from_secs(30))
    .with_compare_contents(true);

    // Automatically select the best implementation for your platform.
    // You can also access each implementation directly e.g. INotifyWatcher.
    let watcher = RecommendedWatcher::new(move |res| {
        futures::executor::block_on(async {
            tx.send(res).await.unwrap();
        })
    }, config)?;

    Ok((watcher, rx))
}

pub async fn watch<P: AsRef<Path>>(path: P, pool:sqlx::Pool<Postgres>) -> notify::Result<()> {
    let (mut watcher, mut rx) = async_watcher()?;

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;

    while let Some(res) = rx.next().await {
        match res {
            Ok(event) => parse_event(event, pool.clone()).await,
            Err(e) => println!("watch error: {:?}", e),
        }
    }

    Ok(())
}

async fn parse_event(event: notify::event::Event,pool:sqlx::Pool<Postgres>) {
    match event.kind {
                                // we sleep here until windows stops messing around with our file smh!
        EventKind::Create(_) => {thread::sleep(time::Duration::from_millis(75));
                                    metadata::scan_file(&event.paths[0], pool).await
                                },
        EventKind::Remove(_) => println!("removed {}", event.paths[0].to_str().unwrap()),
        EventKind::Modify(_) =>(),
        EventKind::Access(_) =>(),
        EventKind::Any => (),
        EventKind::Other => (),
    }
}