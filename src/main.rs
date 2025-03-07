mod gtfs;
mod commands;
use commands::gtfs::GtfsNode;

use commands::CommandInterpreter;
use colored::Colorize;
use curl::easy::Easy;
use std::io;
use std::io::Write;
use std::iter;
use std::io::BufRead;

fn main() {
    let mut buf = Vec::new();

    // open gtfs zip file
    let mut response = Easy::new();
    {
        response.url("https://cdn.mbta.com/MBTA_GTFS.zip").unwrap_or_else(
            |err| panic!("Failed to open gtfs.zip: {}", err)
        );
        response.get(true).unwrap();
        let mut transfer = response.transfer();
        transfer.write_function(|data| {
            buf.extend_from_slice(data);
            pre_log(&format!("Downloaded {} bytes", buf.len()));
            Ok(data.len())
        }).unwrap();
        transfer.perform().unwrap_or_else(
            |err| panic!("Failed to download gtfs.zip: {}", err)
        );
    }

    match response.response_code() {
        Ok(200) => (),
        _ => panic!("Failed to download gtfs.zip: {}", response.response_code().unwrap()),
    }
    pre_log(&format!("Downloaded GTFS feed: {} bytes", buf.len()));

    // interpret as zip archive
    let gtfs_zip = zip::ZipArchive::new(std::io::Cursor::new(buf)).unwrap_or_else(
        |err| panic!("Failed to create zip archive: {}", err)
    );
    // load gtfs feed from archive
    let mut zip_loader = gtfs::loaders::zip_loader::ZipLoader::new(gtfs_zip);
    zip_loader = zip_loader.with_event_handler(gtfs::loaders::zip_loader::FnZipLoaderEventHandler {
        on_stops_file_opened: Box::new(|_| pre_log("Opened stops file")),
        on_stops_loaded: Box::new(|_| pre_log("Loaded stops")),
        on_routes_file_opened: Box::new(|_| pre_log("Opened routes file")),
        on_routes_loaded: Box::new(|_| pre_log("Loaded routes")),
        on_trips_file_opened: Box::new(|_| pre_log("Opened trips file")),
        on_trips_loaded: Box::new(|_| pre_log("Loaded trips")),
        on_stop_times_file_opened: Box::new(|_| pre_log("Opened stop times file")),
        on_stop_times_loaded: Box::new(|_| pre_log("Loaded stop times")),
    });
    let gtfs = zip_loader.load().unwrap_or_else(
        |err| panic!("Failed to create gtfs feed: {}", err)
    );
    pre_log("Loaded gtfs feed");
    println!();

    let stdin = io::stdin();
    print!("> ");
    io::stdout().flush().unwrap();
    for line in stdin.lock().lines() {
        line.map_err(|err| format!("Error reading line: {}", err))
            .and_then(|line| GtfsNode{
                gtfs: gtfs.clone(),
                parent: None,
                node_id: "".to_string(),
                node_name: None
            }.interpret(line.as_str()).map_err(|err| format!("Error interpreting command: {}", err)))
            .unwrap_or_else(|err| println!("{}", err));
        print!("> ");
        io::stdout().flush().unwrap();
    }
}

fn pre_log(message: &str) {
    print!("\r{}", iter::repeat(" ").take(80).collect::<String>());
    print!("\r{}", message.truecolor(128, 128, 128));
    io::stdout().flush().unwrap();
}