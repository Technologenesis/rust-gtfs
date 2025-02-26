mod gtfs;

use std::fs;
use gtfs::routes;
use zip;
use std::collections;
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

    // find all rail lines
    let rail_lines = (&gtfs.routes).into_iter()
        .filter(|route| [
            routes::RouteType::SubwayMetro,
            routes::RouteType::TramStreetcarLightRail,
        ].contains(&route.route_type))
        .map(|route| route.route_id.clone())
        .collect::<collections::HashSet<_>>();

    // get all trips for each rail line
    let trips_by_rail_line = gtfs.trips.into_iter()
        .filter_map(|trip| {
            if rail_lines.contains(&trip.route_id) {
                Some((trip.route_id.clone(), trip.trip_id.clone()))
            } else {
                None
            }
        })
        .collect::<collections::HashMap<_, _>>();

    // get all stops for each rail line
    let stops_by_rail_line = trips_by_rail_line.iter()
        .map(
            |(route_id, trip_id)|
            gtfs.stop_times.stop_times.get(trip_id).map(|stop_times| (route_id, stop_times))
        )
        .filter_map(|opt| opt)
        .map(
            |(route_id, stop_times)|
            (route_id, stop_times.iter()
                .map(|stop_time| &stop_time.stop_id)
                .filter_map(|stop_id| stop_id.as_ref())
                .filter_map(|stop_id| gtfs.stops.stops.get(stop_id))
                .filter_map(|stop| stop.get_stop_name())
                .collect::<collections::HashSet<_>>())
        )
        .collect::<collections::HashMap<_, _>>();

    // print all stops for each rail line
    for (route_id, stops) in stops_by_rail_line {
        let route = gtfs.routes.routes.get(route_id).unwrap();
        println!("{}: {} stops", route.name(), stops.len());
        for stop in stops {
            println!("  {}", stop);
        }
    }
}
