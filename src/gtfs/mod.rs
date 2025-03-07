pub mod stops;
pub mod routes;
pub mod trips;
pub mod stop_times;
pub mod loaders;
use colored::Colorize;

#[derive(Debug, Clone)]
pub struct GtfsSchedule {
    // TODO: fill out remaining fields
    pub stops: stops::Stops,
    pub routes: routes::Routes,
    pub trips: trips::Trips,
    pub stop_times: stop_times::StopTimes,
}



impl std::fmt::Display for GtfsSchedule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}\n{}: {}\n{}: {}",
        "Stops".truecolor(128, 128, 128).bold(), self.stops.stops.len(),
        "Routes".truecolor(128, 128, 128).bold(), self.routes.routes.len(),
        "Trips".truecolor(128, 128, 128).bold(), self.trips.trips.len())
    }
}