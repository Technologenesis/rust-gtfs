pub mod stops;
pub mod routes;
pub mod trips;
pub mod stop_times;
pub mod loaders;
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
