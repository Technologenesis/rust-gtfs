mod gtfs;

use gtfs::routes;
use gtfs::stop_times;
use std::collections;
use colored::Colorize;
use curl::easy::Easy;
use std::io;


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
            print!("\rDownloaded {} bytes", buf.len());
            Ok(data.len())
        }).unwrap();
        transfer.perform().unwrap_or_else(
            |err| panic!("Failed to download gtfs.zip: {}", err)
        );
        print!("\r")
    }

    match response.response_code() {
        Ok(200) => (),
        _ => panic!("Failed to download gtfs.zip: {}", response.response_code().unwrap()),
    }
    println!("Downloaded GTFS feed: {} bytes", buf.len());

    // interpret as zip archive
    let gtfs_zip = zip::ZipArchive::new(std::io::Cursor::new(buf)).unwrap_or_else(
        |err| panic!("Failed to create zip archive: {}", err)
    );
    // load gtfs feed from archive
    let gtfs = gtfs::GtfsSchedule::try_from(gtfs_zip).unwrap_or_else(
        |err| panic!("Failed to create gtfs feed: {}", err)
    );
    println!("Loaded gtfs feed");

    // find all subway, tram, streetcar, light rail, cable tram, or rail lines
    let rail_lines = (&gtfs.routes).into_iter()
        .filter(|route| [
                routes::RouteType::SubwayMetro,
                routes::RouteType::TramStreetcarLightRail,
                routes::RouteType::CableTram,
                routes::RouteType::Rail
            ].contains(&route.route_type))
        .map(|route| route.route_id.clone())
        .collect::<collections::HashSet<_>>();
    println!("Found {} subway, tram, streetcar, light rail, cable tram, or rail lines", rail_lines.len());

    // get all trips for each rail line
    let trips_by_rail_line = gtfs.trips.into_iter()
        .filter_map(|trip| {
            if rail_lines.contains(&trip.route_id) {
                Some((trip.route_id.clone(), trip.trip_id.clone()))
            } else {
                None
            }
        })
        .fold(
            collections::HashMap::new(),
            |mut map, (route_id, trip_id)| {
                map.entry(route_id).or_insert_with(Vec::new).push(trip_id);
                map
            }
        );
    println!("Mapped trips by rail line;");
    // print all trips for each rail line
    for (route_id, trip_ids) in &trips_by_rail_line {
        gtfs.routes.routes.get(route_id)
            .map(
                |route|
                println!(
                    "  {}: {} trips",
                    route.route_color.or(route.route_text_color).map(
                        |color|
                        route.name().truecolor(color.r, color.g, color.b)
                    ).unwrap_or_else(|| colored::ColoredString::from(route.name())).bold(),
                    trip_ids.len()
                )
            );
    }

    // get all stops for each rail line
    let stops_by_rail_line = trips_by_rail_line.iter()
        .map(
            |(route_id, trip_ids)|
            trip_ids.iter().map(|trip_id| gtfs.stop_times.stop_times.get(trip_id).map(|stop_times| (route_id.clone(), stop_times)))
        )
        .flatten()
        .filter_map(|opt| opt.map(
            |(route_id, stop_times)|
            stop_times.iter()
                .map(move |stop_time| (route_id.clone(), stop_time))
        ))
        .flatten()
        .fold(
            collections::HashMap::new(),
            |mut map, (route_id, stop_time)| {
                map
                    .entry(route_id).or_insert_with(collections::HashMap::new)
                    .entry(stop_time.stop_id.clone()).or_insert_with(Vec::new).push(stop_time.clone());
                map
            }
        );

    println!("Mapped stops by rail line");
    // print all stops for each rail line
    for (route_id, stops) in stops_by_rail_line {
        let route = gtfs.routes.routes.get(&route_id).unwrap();
        println!("{}: {} stops", route.route_color.or(route.route_text_color).map(
                |color|
                route.name().truecolor(color.r, color.g, color.b)
            ).unwrap_or_else(|| colored::ColoredString::from(route.name())).bold(), stops.len());
        for (stop, stop_times) in stops
                .into_iter()
                .filter_map(|(stop_id, stop_times)| stop_id.map(|stop_id| (stop_id, stop_times)))
                .filter_map(|(stop_id, stop_time)| gtfs.stops.stops.get(&stop_id).map(|stop| (stop, stop_time))) {
            println!("  {}: {} stop times", stop.get_stop_name().unwrap_or(&format!("Stop ID {}", stop.stop_id)), stop_times.len());
            for stop_time in stop_times.iter().take(5) {
                println!("    {}", stop_time.departure_time.or(stop_time.arrival_time).map(|time| time.format("%H:%M:%S").to_string()).unwrap_or(String::from("unknown stop time")));
            }
            if stop_times.len() > 5 {
                println!("    ...");
            }
        }
        io::stdin().read_line(&mut String::new()).unwrap();
    }
}
