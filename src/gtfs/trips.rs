use csv;
use std::io;
use std::iter;
use std::collections;
use std::fmt;
use std::str::FromStr;

// Trips is a collection of trips, indexed by trip_id.
pub struct Trips {
    pub trips: std::collections::HashMap<String, Trip>
}

impl<'a> iter::IntoIterator for &'a Trips {
    type Item = &'a Trip;
    type IntoIter = std::collections::hash_map::Values<'a, String, Trip>;

    fn into_iter(self) -> Self::IntoIter {
        self.trips.values()
    }
}

impl iter::IntoIterator for Trips {
    type Item = Trip;
    type IntoIter = std::collections::hash_map::IntoValues<String, Trip>;

    fn into_iter(self) -> Self::IntoIter {
        self.trips.into_values()
    }
}

// TripsCsvLoadError is an error that occurs when loading trips from a CSV file.
pub enum TripsCsvLoadError {
    NoHeader,
    TripLoadError(TripLoadError),
    CSVReadError(csv::Error)
}

impl fmt::Display for TripsCsvLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoHeader => write!(f, "No header found"),
            Self::TripLoadError(e) => write!(f, "Error loading trip: {}", e),
            Self::CSVReadError(e) => write!(f, "Error reading CSV: {}", e)
        }
    }
}

// Trips implements TryFrom<csv::Reader<R>> by attempting to consume and read from a csv::Reader<R>.
impl<R: io::Read> TryFrom<csv::Reader<R>> for Trips {
    // The error type for this function is TripsCsvLoadError.
    type Error = TripsCsvLoadError;

    // try_from consumes the csv::Reader<R> and returns a Result holding a Routes object, or a RoutesCsvLoadError.
    fn try_from(mut r: csv::Reader<R>) -> Result<Self, Self::Error> {
        // try to get the headers; if there are no headers, return a TripsCsvLoadError::NoHeader.
        r.headers().cloned().map_err(|_| TripsCsvLoadError::NoHeader).and_then(
            // if there are headers, try to create a Trips object from the remaining records.
            |header|
            Ok(Trips {
                // to create the actual collection of trips, we need to iterate over the records
                trips: r.into_records()
                    // and fold them into an overarching result containing the collection.
                    .fold(
                        Ok(collections::HashMap::new()),
                        // at each stage of the fold,
                        |trips_result, record_result|
                        // proceed only if the running result is Ok.
                        trips_result
                        .and_then(|mut trips|
                            // if there was an error reading this record, return that error.
                            record_result.map_err(TripsCsvLoadError::CSVReadError)
                                .and_then(
                                    // otherwise, try to create a Trip object from the record.
                                    |record|
                                    Trip::try_from(
                                        // Zip the header and record together,
                                        iter::zip(
                                            header.iter().map(|s| s.to_string()),
                                            record.iter().map(|s| s.to_string())
                                        )
                                        // and collect the results into a HashMap.
                                        .collect::<collections::HashMap<String, String>>()
                                    )
                                    // if there is an error creating the Trip object from the HashMap, return that error.
                                    .map_err(|err| TripsCsvLoadError::TripLoadError(err))
                            )
                            .map(
                                |trip| {
                                    // insert the Trip object into the HashMap.
                                    trips.insert(trip.trip_id.clone(), trip);
                                    // return the updated HashMap.
                                    trips
                                }
                            )
                        )
                    // extract the HashMap from the Result, or return the error.
                    )?
            })
        )
    }
}

#[derive(Debug)]
pub struct Trip {
    pub trip_id: String,
    pub route_id: String,
    pub service_id: String,
    pub trip_headsign: Option<String>,
    pub trip_short_name: Option<String>,
    pub direction_id: Option<Direction>,
    pub block_id: Option<String>,
    pub shape_id: Option<String>,
    pub wheelchair_accessible: Option<bool>,
    pub bikes_allowed: Option<bool>,
}

// represents two arbitrary opposing directions
#[derive(Debug)]
pub enum Direction {
    A,
    B
}

impl FromStr for Direction {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "0" => Ok(Direction::A),
            "1" => Ok(Direction::B),
            _ => Err(format!("invalid direction '{}'", s))
        }
    }
}

pub enum TripLoadError {
    TripIdRequired,
    RouteIdRequired,
    ServiceIdRequired,
    TripHeadsignError(String),
    TripShortNameError(String),
    DirectionIdError(String),
    BlockIdError(String),
    ShapeIdRequired,
    WheelchairAccessibleError(String),
    BikesAllowedError(String),
}

impl fmt::Display for TripLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TripIdRequired => write!(f, "trip_id is required"),
            Self::RouteIdRequired => write!(f, "route_id is required"),
            Self::ServiceIdRequired => write!(f, "service_id is required"),
            Self::TripHeadsignError(e) => write!(f, "Error parsing trip headsign: {}", e),
            Self::TripShortNameError(e) => write!(f, "Error parsing trip short name: {}", e),
            Self::DirectionIdError(e) => write!(f, "Error parsing direction id: {}", e),
            Self::BlockIdError(e) => write!(f, "Error parsing block id: {}", e),
            Self::ShapeIdRequired => write!(f, "shape_id is required"),
            Self::WheelchairAccessibleError(e) => write!(f, "Error parsing wheelchair accessible: {}", e),
            Self::BikesAllowedError(e) => write!(f, "Error parsing bikes allowed: {}", e),
        }
    }
}

// Route implements TryFrom<collections::HashMap<String, String>> by interpreting the keys as field names, and
// the values as string-encoded values for those fields.
impl TryFrom<collections::HashMap<String, String>> for Trip {
    type Error = TripLoadError;

    fn try_from(fields: collections::HashMap<String, String>) -> Result<Self, Self::Error> {
        Ok(Trip {
            trip_id: fields.get("trip_id")
                .filter(|s| !s.is_empty())
                .ok_or(TripLoadError::TripIdRequired)?
                .clone(),
            route_id: fields.get("route_id")
                .filter(|s| !s.is_empty())
                .ok_or(TripLoadError::RouteIdRequired)?
                .clone(),
            service_id: fields.get("service_id")
                .filter(|s| !s.is_empty())
                .ok_or(TripLoadError::ServiceIdRequired)?
                .clone(),
            trip_headsign: fields.get("trip_headsign").filter(|s| !s.is_empty()).cloned(),
            trip_short_name: fields.get("trip_short_name").filter(|s| !s.is_empty()).cloned(),
            direction_id: match fields.get("direction_id")
                .filter(|s| !s.is_empty())
                .map(|s| s.parse::<Direction>())
            {
                Some(Ok(direction)) => Some(direction),
                Some(Err(e)) => return Err(TripLoadError::DirectionIdError(e)),
                None => None
            },
            block_id: fields.get("block_id").filter(|s| !s.is_empty()).cloned(),
            shape_id: fields.get("shape_id")
                .filter(|s| !s.is_empty())
                .cloned(),
            wheelchair_accessible: match fields.get("wheelchair_accessible")
                    .filter(|s| !s.is_empty())
                {
                    None => Ok(None),
                    Some(s) => match s.as_str() {
                        "0" => Ok(None),
                        "1" => Ok(Some(true)),
                        "2" => Ok(Some(false)),
                        _ => Err(TripLoadError::WheelchairAccessibleError(s.clone()))
                    }
                }?,
            bikes_allowed: match fields.get("bikes_allowed")
                    .filter(|s| !s.is_empty()) 
                {
                    None => Ok(None),
                    Some(s) => match s.as_str() {
                        "0" => Ok(None),
                        "1" => Ok(Some(true)),
                        "2" => Ok(Some(false)),
                        _ => Err(TripLoadError::BikesAllowedError(s.clone()))
                    }
                }?,
        })
    }
}