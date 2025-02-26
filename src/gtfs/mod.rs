pub mod stops;
pub mod routes;
pub mod trips;
pub mod stop_times;

use zip;
use zip::result::ZipError;
use csv;
use std::io;
use std::fmt;

pub struct GtfsSchedule {
    // TODO: fill out remaining fields
    pub stops: stops::Stops,
    pub routes: routes::Routes,
    pub trips: trips::Trips,
    pub stop_times: stop_times::StopTimes,
}

pub enum GtfsScheduleLoadError {
    FailedToOpenStops(String, ZipError),
    FailedToOpenRoutes(String, ZipError),
    FailedToOpenTrips(String, ZipError),
    FailedToOpenStopTimes(String, ZipError),
    FailedToLoadStops(stops::StopsCsvLoadError),
    FailedToLoadRoutes(routes::RoutesCsvLoadError),
    FailedToLoadTrips(trips::TripsCsvLoadError),
    FailedToLoadStopTimes(stop_times::StopTimesCsvLoadError),
}

impl fmt::Display for GtfsScheduleLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FailedToOpenStops(file, e) => write!(f, "Failed to open {}: {}", file, e),
            Self::FailedToOpenRoutes(file, e) => write!(f, "Failed to open {}: {}", file, e),
            Self::FailedToOpenTrips(file, e) => write!(f, "Failed to open {}: {}", file, e),
            Self::FailedToOpenStopTimes(file, e) => write!(f, "Failed to open {}: {}", file, e),
            Self::FailedToLoadStops(e) => write!(f, "Failed to load stops: {}", e),
            Self::FailedToLoadRoutes(e) => write!(f, "Failed to load routes: {}", e),
            Self::FailedToLoadTrips(e) => write!(f, "Failed to load trips: {}", e),
            Self::FailedToLoadStopTimes(e) => write!(f, "Failed to load stop times: {}", e),
        }
    }
}

impl<R: io::Read + io::Seek> TryFrom<zip::ZipArchive<R>> for GtfsSchedule {
    type Error = GtfsScheduleLoadError;

    fn try_from(mut archive: zip::ZipArchive<R>) -> Result<Self, Self::Error> {
        let stops_reader = archive.by_name("stops.txt")
            .map_err(
                |e|
                GtfsScheduleLoadError::FailedToOpenStops("stops.txt".to_string(), e)
            )?;
        
        let stops = stops::Stops::try_from(csv::Reader::from_reader(stops_reader))
        .map_err(
            |e|
            GtfsScheduleLoadError::FailedToLoadStops(e)
        )?;
        
        let routes_reader = archive.by_name("routes.txt")
            .map_err(
                |e|
                GtfsScheduleLoadError::FailedToOpenRoutes("routes.txt".to_string(), e)
            )?;

        let routes = routes::Routes::try_from(csv::Reader::from_reader(routes_reader))
            .map_err(
                |e|
                GtfsScheduleLoadError::FailedToLoadRoutes(e)
            )?;

        let trips_reader = archive.by_name("trips.txt")
            .map_err(
                |e|
                GtfsScheduleLoadError::FailedToOpenTrips("trips.txt".to_string(), e)
            )?;

        let trips = trips::Trips::try_from(csv::Reader::from_reader(trips_reader))
            .map_err(
                |e|
                GtfsScheduleLoadError::FailedToLoadTrips(e)
            )?;

        let stop_times_reader = archive.by_name("stop_times.txt")
            .map_err(
                |e|
                GtfsScheduleLoadError::FailedToOpenStopTimes("stop_times.txt".to_string(), e)
            )?;

        let stop_times = stop_times::StopTimes::try_from(csv::Reader::from_reader(stop_times_reader))
            .map_err(
                |e|
                GtfsScheduleLoadError::FailedToLoadStopTimes(e)
            )?;
        
        Ok(Self {
            stops,
            routes,
            trips,
            stop_times,
        })
    }
}
