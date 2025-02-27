use crate::gtfs;
use crate::gtfs::stops;
use crate::gtfs::routes;
use crate::gtfs::trips;
use crate::gtfs::stop_times;
use zip::read::ZipFile;
use zip::result::ZipError;
use std::fmt;

pub struct ZipLoader<Handler: ZipLoaderEventHandler> {
    pub zip: zip::ZipArchive<std::io::Cursor<Vec<u8>>>,
    pub event_handler: Handler,
}


pub enum ZipLoaderError {
    FailedToOpenStops(String, ZipError),
    FailedToOpenRoutes(String, ZipError),
    FailedToOpenTrips(String, ZipError),
    FailedToOpenStopTimes(String, ZipError),
    FailedToLoadStops(stops::StopsCsvLoadError),
    FailedToLoadRoutes(routes::RoutesCsvLoadError),
    FailedToLoadTrips(trips::TripsCsvLoadError),
    FailedToLoadStopTimes(stop_times::StopTimesCsvLoadError),
}

impl fmt::Display for ZipLoaderError {
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

impl ZipLoader<FnZipLoaderEventHandler> {
    pub fn new(zip: zip::ZipArchive<std::io::Cursor<Vec<u8>>>) -> Self {
        Self {
            zip,
            event_handler: noop_handler(),
        }
    }
}

impl<Handler: ZipLoaderEventHandler> ZipLoader<Handler> {
    pub fn with_event_handler<NewHandler: ZipLoaderEventHandler>(self, event_handler: NewHandler) -> ZipLoader<NewHandler> {
        ZipLoader {
            zip: self.zip,
            event_handler,
        }
    }

    pub fn load(&mut self) -> Result<gtfs::GtfsSchedule, ZipLoaderError> {
        let stops_reader = self.zip.by_name("stops.txt")
            .map_err(
                |e|
                ZipLoaderError::FailedToOpenStops("stops.txt".to_string(), e)
            )?;
        self.event_handler.on_stops_file_opened(&stops_reader);
        
        let stops = stops::Stops::try_from(csv::Reader::from_reader(stops_reader))
        .map_err(
            |e|
            ZipLoaderError::FailedToLoadStops(e)
        )?;
        self.event_handler.on_stops_loaded(&stops);
        let routes_reader = self.zip.by_name("routes.txt")
            .map_err(
                |e|
                ZipLoaderError::FailedToOpenRoutes("routes.txt".to_string(), e)
            )?;
        self.event_handler.on_routes_file_opened(&routes_reader);
        let routes = routes::Routes::try_from(csv::Reader::from_reader(routes_reader))
            .map_err(
                |e|
                ZipLoaderError::FailedToLoadRoutes(e)
            )?;
        self.event_handler.on_routes_loaded(&routes);

        let trips_reader = self.zip.by_name("trips.txt")
            .map_err(
                |e|
                ZipLoaderError::FailedToOpenTrips("trips.txt".to_string(), e)
            )?;
        self.event_handler.on_trips_file_opened(&trips_reader);

        let trips = trips::Trips::try_from(csv::Reader::from_reader(trips_reader))
            .map_err(
                |e|
                ZipLoaderError::FailedToLoadTrips(e)
            )?;
        self.event_handler.on_trips_loaded(&trips);

        let stop_times_reader = self.zip.by_name("stop_times.txt")
            .map_err(
                |e|
                ZipLoaderError::FailedToOpenStopTimes("stop_times.txt".to_string(), e)
            )?;
        self.event_handler.on_stop_times_file_opened(&stop_times_reader);

        let stop_times = stop_times::StopTimes::try_from(csv::Reader::from_reader(stop_times_reader))
            .map_err(
                |e|
                ZipLoaderError::FailedToLoadStopTimes(e)
            )?;
        self.event_handler.on_stop_times_loaded(&stop_times);

        Ok(gtfs::GtfsSchedule {
            stops,
            routes,
            trips,
            stop_times,
        })
    }
}

trait ZipLoaderEventHandler {
    fn on_stops_file_opened(&self, stops_reader: &ZipFile);
    fn on_stops_loaded(&self, stops: &gtfs::stops::Stops);
    fn on_routes_file_opened(&self, routes_reader: &ZipFile);
    fn on_routes_loaded(&self, routes: &gtfs::routes::Routes);
    fn on_trips_file_opened(&self, trips_reader: &ZipFile);
    fn on_trips_loaded(&self, trips: &gtfs::trips::Trips);
    fn on_stop_times_file_opened(&self, stop_times_reader: &ZipFile);
    fn on_stop_times_loaded(&self, stop_times: &gtfs::stop_times::StopTimes);
}

pub struct FnZipLoaderEventHandler {
    pub on_stops_file_opened: Box<dyn Fn(&ZipFile)>,
    pub on_stops_loaded: Box<dyn Fn(&gtfs::stops::Stops)>,
    pub on_routes_file_opened: Box<dyn Fn(&ZipFile)>,
    pub on_routes_loaded: Box<dyn Fn(&gtfs::routes::Routes)>,
    pub on_trips_file_opened: Box<dyn Fn(&ZipFile)>,
    pub on_trips_loaded: Box<dyn Fn(&gtfs::trips::Trips)>,
    pub on_stop_times_file_opened: Box<dyn Fn(&ZipFile)>,
    pub on_stop_times_loaded: Box<dyn Fn(&gtfs::stop_times::StopTimes)>
}

fn noop_handler() -> FnZipLoaderEventHandler {
    FnZipLoaderEventHandler {
        on_stops_file_opened: Box::new(|_| ()),
        on_stops_loaded: Box::new(|_| ()),
        on_routes_file_opened: Box::new(|_| ()),
        on_routes_loaded: Box::new(|_| ()),
        on_trips_file_opened: Box::new(|_| ()),
        on_trips_loaded: Box::new(|_| ()),
        on_stop_times_file_opened: Box::new(|_| ()),
        on_stop_times_loaded: Box::new(|_| ()),
    }
}

impl ZipLoaderEventHandler for FnZipLoaderEventHandler {
    fn on_stops_file_opened(&self, stops_reader: &ZipFile) {
        (self.on_stops_file_opened)(stops_reader);
    }

    fn on_stops_loaded(&self, stops: &gtfs::stops::Stops) {
        (self.on_stops_loaded)(stops);
    }

    fn on_routes_file_opened(&self, routes_reader: &ZipFile) {
        (self.on_routes_file_opened)(routes_reader);
    }

    fn on_routes_loaded(&self, routes: &gtfs::routes::Routes) {
        (self.on_routes_loaded)(routes);
    }

    fn on_trips_file_opened(&self, trips_reader: &ZipFile) {
        (self.on_trips_file_opened)(trips_reader);
    }

    fn on_trips_loaded(&self, trips: &gtfs::trips::Trips) {
        (self.on_trips_loaded)(trips);
    }

    fn on_stop_times_file_opened(&self, stop_times_reader: &ZipFile) {
        (self.on_stop_times_file_opened)(stop_times_reader);
    }

    fn on_stop_times_loaded(&self, stop_times: &gtfs::stop_times::StopTimes) {
        (self.on_stop_times_loaded)(stop_times);
    }
}