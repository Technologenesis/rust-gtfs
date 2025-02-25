mod gtfs;

use std::fs;
use zip;

fn main() {
    // open gtfs zip file
    let gtfs_file = fs::File::open("data/gtfs.zip").unwrap_or_else(
        |err| panic!("Failed to open gtfs.zip: {}; CWD: {}", err, std::env::current_dir().unwrap().display())
    );

    // interpret as zip archive
    let gtfs_zip = zip::ZipArchive::new(gtfs_file).unwrap_or_else(
        |err| panic!("Failed to create zip archive: {}", err)
    );

    // load gtfs feed from archive
    let gtfs = gtfs::GtfsSchedule::try_from(gtfs_zip).unwrap_or_else(
        |err| panic!("Failed to create gtfs feed: {}", err)
    );

    // iterate over stops
    /*
    (&gtfs.stops).into_iter()
        // filter by stops with "heath st" in the name
        // and map each result to the stop name
        .filter_map(
            |stop|
            stop.get_stop_name().filter(
                |name|
                name.to_lowercase().contains("heath st")
            )
        )
        // log each resulting stop name
        .for_each(
            |stop_name|
            println!("{:?}", stop_name)
        )
    */

    // iterate over routes
    let e_line = gtfs.routes.into_iter().find(|route| route.name() == "Green Line E")
        .unwrap_or_else(|| panic!("Green Line E not found"));

    let e_line_trips = gtfs.trips.into_iter().filter(|trip| trip.route_id == e_line.route_id);

    for trip in e_line_trips {
        println!("{:?}\n", trip);
    }
}
