use csv;
use std::io;
use std::iter;
use std::collections;
use std::fmt;
use std::str::FromStr;

use chrono;
use crate::gtfs::routes;

// StopTimes is a collection of stop times, indexed by trip_id.
pub struct StopTimes {
    pub stop_times: std::collections::HashMap<String, Vec<StopTime>>
}

impl StopTimes {
    fn iter(&self) -> impl Iterator<Item = &StopTime> {
        self.stop_times.values().map(<&Vec<StopTime>>::into_iter).flatten()
    }
}

// StopTimesCsvLoadError is an error that occurs when loading stop times from a CSV file.
pub enum StopTimesCsvLoadError {
    NoHeader,
    StopTimeLoadError(StopTimeLoadError),
    CSVReadError(csv::Error)
}

impl fmt::Display for StopTimesCsvLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoHeader => write!(f, "No header found"),
            Self::StopTimeLoadError(e) => write!(f, "Error loading stop time: {}", e),
            Self::CSVReadError(e) => write!(f, "Error reading CSV: {}", e)
        }
    }
}

// Trips implements TryFrom<csv::Reader<R>> by attempting to consume and read from a csv::Reader<R>.
impl<R: io::Read> TryFrom<csv::Reader<R>> for StopTimes {
    // The error type for this function is StopTimesCsvLoadError.
    type Error = StopTimesCsvLoadError;

    // try_from consumes the csv::Reader<R> and returns a Result holding a Routes object, or a RoutesCsvLoadError.
    fn try_from(mut r: csv::Reader<R>) -> Result<Self, Self::Error> {
        // try to get the headers; if there are no headers, return a StopTimesCsvLoadError::NoHeader.
        r.headers().cloned().map_err(|_| StopTimesCsvLoadError::NoHeader).and_then(
            // if there are headers, try to create a StopTimes object from the remaining records.
            |header|
            Ok(StopTimes {
                // to create the actual collection of stop times, we need to iterate over the records
                stop_times: r.into_records()
                    // and fold them into an overarching result containing the collection.
                    .fold(
                        Ok(collections::HashMap::new()),
                        // at each stage of the fold,
                        |stop_times_result, record_result|
                        // proceed only if the running result is Ok.
                        stop_times_result
                        .and_then(|mut stop_times|
                            // if there was an error reading this record, return that error.
                            record_result.map_err(StopTimesCsvLoadError::CSVReadError)
                                .and_then(
                                    // otherwise, try to create a StopTime object from the record.
                                    |record|
                                    StopTime::try_from(
                                        // Zip the header and record together,
                                        iter::zip(
                                            header.iter().map(|s| s.to_string()),
                                            record.iter().map(|s| s.to_string())
                                        )
                                        // and collect the results into a HashMap.
                                        .collect::<collections::HashMap<String, String>>()
                                    )
                                    // if there is an error creating the StopTime object from the HashMap, return that error.
                                    .map_err(|err| StopTimesCsvLoadError::StopTimeLoadError(err))
                            )
                            .map(
                                |stop_time| {
                                    // insert the StopTime object into the HashMap.
                                    stop_times.get_mut(&stop_time.trip_id)
                                        .map(|v: &mut Vec<StopTime>| v.push(stop_time));
                                    // return the updated HashMap.
                                    stop_times
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
pub struct StopTime {
    pub trip_id: String,
    pub stop_id: Option<String>,
    pub arrival_time: Option<chrono::NaiveDateTime>,
    pub departure_time: Option<chrono::NaiveDateTime>,
    pub location_group_id: Option<String>,
    pub location_id: Option<String>,
    pub stop_sequence: usize,
    pub stop_headsign: Option<String>,
    pub start_pickup_drop_off_window: Option<chrono::NaiveDateTime>,
    pub end_pickup_drop_off_window: Option<chrono::NaiveDateTime>,
    pub pickup_type: Option<StopPolicy>,
    pub drop_off_type: Option<StopPolicy>,
    pub continuous_pickup: Option<routes::RouteContinuityPolicy>,
    pub continuous_drop_off: Option<routes::RouteContinuityPolicy>,
    pub shape_dist_traveled: Option<f64>,
    pub timepoint: Option<Timepoint>,
    pub pickup_booking_rule_id: Option<String>,
    pub drop_off_booking_rule_id: Option<String>,
}

#[derive(Debug)]
pub enum StopPolicy {
    RegularlyScheduled,
    Unavailable,
    Prearrange,
    CoordinateWithDriver,
}

#[derive(Debug)]
pub enum Timepoint {
    Approximate,
    Exact,
}

pub enum StopTimeLoadError {
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

impl fmt::Display for StopTimeLoadError {
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
impl TryFrom<collections::HashMap<String, String>> for StopTime {
    type Error = StopTimeLoadError;

    fn try_from(fields: collections::HashMap<String, String>) -> Result<Self, Self::Error> {
        Ok(StopTime {
            trip_id: fields.remove("trip_id")
                .filter(|s| !s.is_empty())
                .ok_or(StopTimeLoadError::TripIdRequired)?,
            stop_id: fields.remove("stop_id")
                .filter(|s| !s.is_empty()),
            arrival_time: match
                fields.remove("arrival_time")
                .filter(|s| !s.is_empty()) {
                    Some(s) => chrono::NaiveTime::from_str(&s)
                        .map_err(StopTimeLoadError::ArrivalTimeError)
                        .map(|t| Some(t)),
                    None => Ok(None),
                }?,
            departure_time: match
                fields.remove("departure_time")
                .filter(|s| !s.is_empty()) {
                    Some(s) => chrono::NaiveTime::from_str(&s)
                        .map_err(StopTimeLoadError::DepartureTimeError)
                        .map(|t| Some(t)),
                    None => Ok(None),
                }?,
            location_group_id: fields.remove("location_group_id")
                .filter(|s| !s.is_empty()),
            location_id: fields.remove("location_id")
                .filter(|s| !s.is_empty()),
            stop_sequence: fields.remove("stop_sequence")
                .filter(|s| !s.is_empty())
                .ok_or(StopTimeLoadError::StopSequenceRequired)?
                .parse::<usize>()?,
            stop_headsign: fields.remove("stop_headsign")
                .filter(|s| !s.is_empty()),
            start_pickup_drop_off_window: match
                fields.remove("start_pickup_drop_off_window")
                .filter(|s| !s.is_empty()) {
                    Some(s) => chrono::NaiveDateTime::from_str(&s)
                        .map_err(StopTimeLoadError::StartPickupDropOffWindowError)
                        .map(|t| Some(t)),
                    None => Ok(None),
                }?,
            end_pickup_drop_off_window: match
                fields.remove("end_pickup_drop_off_window")
                .filter(|s| !s.is_empty()) {
                    Some(s) => chrono::NaiveDateTime::from_str(&s)
                        .map_err(StopTimeLoadError::EndPickupDropOffWindowError)
                        .map(|t| Some(t)),
                    None => Ok(None),
                }?,
            pickup_type: fields.remove("pickup_type")
                .filter(|s| !s.is_empty())
                .map(|s| s.parse::<StopPolicy>())
                .and_then(|p| p.map_err(StopTimeLoadError::PickupTypeError))
                .ok(),
            drop_off_type: fields.get("drop_off_type")
                .filter(|s| !s.is_empty())
                .map(|s| s.parse::<StopPolicy>())
                .and_then(|p| p.map_err(StopTimeLoadError::DropOffTypeError))
                .ok(),
            continuous_pickup: fields.get("continuous_pickup")
                .filter(|s| !s.is_empty())
                .map(|s| s.parse::<routes::RouteContinuityPolicy>())
                .and_then(|p| p.map_err(StopTimeLoadError::ContinuousPickupError))
                .ok(),
            continuous_drop_off: fields.get("continuous_drop_off")
                .filter(|s| !s.is_empty())
                .map(|s| s.parse::<routes::RouteContinuityPolicy>())
                .and_then(|p| p.map_err(StopTimeLoadError::ContinuousDropOffError))
                .ok(),
            shape_dist_traveled: fields.get("shape_dist_traveled")
                .filter(|s| !s.is_empty())
                .map(|s| s.parse::<f64>())
                .and_then(|p| p.map_err(StopTimeLoadError::ShapeDistTraveledError))
                .ok(),
            wheelchair_accessible: fields.get("wheelchair_accessible")
                .filter(|s| !s.is_empty())
                .map(|s| s.parse::<bool>())
                .and_then(|p| p.map_err(StopTimeLoadError::WheelchairAccessibleError))
                .ok(),
            bikes_allowed: fields.get("bikes_allowed")
                .filter(|s| !s.is_empty())
                .map(|s| s.parse::<bool>())
                .and_then(|p| p.map_err(StopTimeLoadError::BikesAllowedError))
                .ok(),
            pickup_booking_rule_id: fields.get("pickup_booking_rule_id")
                .filter(|s| !s.is_empty())
                .map(|s| s.parse::<String>())
                .and_then(|p| p.map_err(StopTimeLoadError::PickupBookingRuleIdError))
                .ok(),
            drop_off_booking_rule_id: fields.get("drop_off_booking_rule_id")
                .filter(|s| !s.is_empty())
                .map(|s| s.parse::<String>())
                .and_then(|p| p.map_err(StopTimeLoadError::DropOffBookingRuleIdError))
                .ok(),
            timepoint: fields.get("timepoint")
                .filter(|s| !s.is_empty())
                .map(|s| s.parse::<Timepoint>())
                .and_then(|p| p.map_err(StopTimeLoadError::TimepointError))
                .ok(),
        })
    }
}