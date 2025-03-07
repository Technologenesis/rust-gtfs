use csv;
use std::f32::consts::E;
use std::io;
use std::iter;
use std::collections;
use std::fmt;
use std::str::FromStr;
use std::num;
use chrono;
use crate::gtfs::routes;

// StopTimes is a collection of stop times, indexed by trip_id.
#[derive(Debug, Clone)]
pub struct StopTimes {
    pub stop_times: std::collections::HashMap<String, Vec<StopTime>>
}

impl StopTimes {
    pub fn iter(&self) -> impl Iterator<Item = &StopTime> {
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
                    .try_fold(
                        collections::HashMap::new(),
                        // at each stage of the fold,
                        |mut stop_times, record_result|
                            // if there was an error reading this record, return that error.
                            record_result.map_err(StopTimesCsvLoadError::CSVReadError)
                                .and_then(
                                    // otherwise, try to create a StopTime object from the record.
                                    |record|
                                    StopTime::try_from(
                                        // Zip the header and record together,
                                        &(iter::zip(
                                            header.iter().map(|s| s.to_string()),
                                            record.iter().map(|s| s.to_string())
                                        )
                                        // and collect the results into a HashMap.
                                        .collect::<collections::HashMap<String, String>>())
                                    )
                                    // if there is an error creating the StopTime object from the HashMap, return that error.
                                    .map_err(|err| StopTimesCsvLoadError::StopTimeLoadError(err))
                            )
                            .map(
                                |stop_time| {
                                    // insert the StopTime object into the HashMap.
                                    stop_times.get_mut(&stop_time.trip_id)
                                        .map(|v: &mut Vec<StopTime>| v.push(stop_time.clone()))
                                        .unwrap_or_else(|| {
                                            stop_times.insert(stop_time.trip_id.clone(), vec![stop_time]);
                                        });
                                    // return the updated HashMap.
                                    stop_times
                                }
                            )
                    // extract the HashMap from the Result, or return the error.
                    )?
            })
        )
    }
}

#[derive(Debug, Clone)]
pub struct StopTime {
    pub trip_id: String,
    pub stop_id: Option<String>,
    pub arrival_time: Option<chrono::NaiveTime>,
    pub departure_time: Option<chrono::NaiveTime>,
    pub location_group_id: Option<String>,
    pub location_id: Option<String>,
    pub stop_sequence: usize,
    pub stop_headsign: Option<String>,
    pub start_pickup_drop_off_window: Option<chrono::NaiveTime>,
    pub end_pickup_drop_off_window: Option<chrono::NaiveTime>,
    pub pickup_type: Option<StopPolicy>,
    pub drop_off_type: Option<StopPolicy>,
    pub continuous_pickup: Option<routes::RouteContinuityPolicy>,
    pub continuous_drop_off: Option<routes::RouteContinuityPolicy>,
    pub shape_dist_traveled: Option<f64>,
    pub timepoint: Option<Timepoint>,
    pub pickup_booking_rule_id: Option<String>,
    pub drop_off_booking_rule_id: Option<String>,
}

#[derive(Debug, Clone)]
pub enum StopPolicy {
    RegularlyScheduled,
    Unavailable,
    Prearrange,
    CoordinateWithDriver,
}

#[derive(Debug)]
pub enum StopPolicyLoadError {
    InvalidStopPolicy(String),
}

impl fmt::Display for StopPolicyLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidStopPolicy(s) => write!(f, "Invalid stop policy: {}", s),
        }
    }
}

impl FromStr for StopPolicy {
    type Err = StopPolicyLoadError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "0" => StopPolicy::RegularlyScheduled,
            "1" => StopPolicy::Unavailable,
            "2" => StopPolicy::Prearrange,
            "3" => StopPolicy::CoordinateWithDriver,
            _ => return Err(StopPolicyLoadError::InvalidStopPolicy(s.to_string())),
        })
    }
}

#[derive(Debug, Clone)]
pub enum Timepoint {
    Approximate,
    Exact,
}

pub enum TimepointLoadError {
    InvalidTimepoint(String),
}

impl fmt::Display for TimepointLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTimepoint(s) => write!(f, "Invalid timepoint: {}", s),
        }
    }
}

impl FromStr for Timepoint {
    type Err = TimepointLoadError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "0" => Timepoint::Approximate,
            "1" => Timepoint::Exact,
            _ => return Err(TimepointLoadError::InvalidTimepoint(s.to_string())),
        })
    }
}


pub enum StopTimeLoadError {
    TripIdRequired,
    ArrivalTimeError(ParseTimeError),
    DepartureTimeError(ParseTimeError),
    StopSequenceRequired,
    StopSequenceError(num::ParseIntError),
    StartPickupDropOffWindowError(ParseTimeError),
    EndPickupDropOffWindowError(ParseTimeError) ,
    PickupTypeError(StopPolicyLoadError),
    DropOffTypeError(StopPolicyLoadError),
    ContinuousPickupError(routes::RouteContinuityPolicyLoadError),
    ContinuousDropOffError(routes::RouteContinuityPolicyLoadError),
    ShapeDistTraveledError(num::ParseFloatError),
    TimepointError(TimepointLoadError),
}

impl fmt::Display for StopTimeLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TripIdRequired => write!(f, "trip_id is required"),
            Self::ArrivalTimeError(e) => write!(f, "Error parsing arrival time: {}", e),
            Self::DepartureTimeError(e) => write!(f, "Error parsing departure time: {}", e),
            Self::StopSequenceRequired => write!(f, "stop_sequence is required"),
            Self::StopSequenceError(e) => write!(f, "Error parsing stop sequence: {}", e),
            Self::StartPickupDropOffWindowError(e) => write!(f, "Error parsing start pickup drop off window: {}", e),
            Self::EndPickupDropOffWindowError(e) => write!(f, "Error parsing end pickup drop off window: {}", e),
            Self::PickupTypeError(e) => write!(f, "Error parsing pickup type: {}", e),
            Self::DropOffTypeError(e) => write!(f, "Error parsing drop off type: {}", e),
            Self::ContinuousPickupError(e) => write!(f, "Error parsing continuous pickup: {}", e),
            Self::ContinuousDropOffError(e) => write!(f, "Error parsing continuous drop off: {}", e),
            Self::ShapeDistTraveledError(e) => write!(f, "Error parsing shape dist traveled: {}", e),
            Self::TimepointError(e) => write!(f, "Error parsing timepoint: {}", e),
        }
    }
}

// Route implements TryFrom<collections::HashMap<String, String>> by interpreting the keys as field names, and
// the values as string-encoded values for those fields.
impl TryFrom<&collections::HashMap<String, String>> for StopTime {
    type Error = StopTimeLoadError;

    fn try_from(fields: &collections::HashMap<String, String>) -> Result<Self, Self::Error> {
        Ok(StopTime {
            trip_id: fields.get("trip_id")
                .filter(|s| !s.is_empty())
                .ok_or(StopTimeLoadError::TripIdRequired)
                .cloned()?,
            stop_id: fields.get("stop_id")
                .filter(|s| !s.is_empty())
                .cloned(),
            arrival_time: fields.get("arrival_time")
                .filter(|s| !s.is_empty())
                .map(|s| parse_time(&s))
                .transpose()
                .map_err(|e| StopTimeLoadError::ArrivalTimeError(e))?,
            departure_time: fields.get("departure_time")
                .filter(|s| !s.is_empty())
                .map(|s| parse_time(&s))
                .transpose()
                .map_err(|e| StopTimeLoadError::DepartureTimeError(e))?,
            location_group_id: fields.get("location_group_id")
                .filter(|s| !s.is_empty())
                .cloned(),
            location_id: fields.get("location_id")
                .filter(|s| !s.is_empty())
                .cloned(),
            stop_sequence: fields.get("stop_sequence")
                .filter(|s| !s.is_empty())
                .ok_or(StopTimeLoadError::StopSequenceRequired)?
                .parse::<usize>()
                .map_err(StopTimeLoadError::StopSequenceError)?,
            stop_headsign: fields.get("stop_headsign")
                .filter(|s| !s.is_empty())
                .cloned(),
            start_pickup_drop_off_window: fields.get("start_pickup_drop_off_window")
                .filter(|s| !s.is_empty())
                .map(|s| parse_time(&s))
                .transpose()
                .map_err(|e| StopTimeLoadError::StartPickupDropOffWindowError(e))?,
            end_pickup_drop_off_window: fields.get("end_pickup_drop_off_window")
                .filter(|s| !s.is_empty())
                .map(|s| parse_time(&s))
                .transpose()
                .map_err(|e| StopTimeLoadError::EndPickupDropOffWindowError(e))?,
            pickup_type: fields.get("pickup_type")
                .filter(|s| !s.is_empty())
                .map(|s| s.parse::<StopPolicy>())
                .transpose()
                .map_err(StopTimeLoadError::PickupTypeError)?,
            drop_off_type: fields.get("drop_off_type")
                .filter(|s| !s.is_empty())
                .map(|s| s.parse::<StopPolicy>())
                .transpose()
                .map_err(StopTimeLoadError::DropOffTypeError)?,
            continuous_pickup: fields.get("continuous_pickup")
                .filter(|s| !s.is_empty())
                .map(|s| s.parse::<routes::RouteContinuityPolicy>())
                .transpose()
                .map_err(StopTimeLoadError::ContinuousPickupError)?,
            continuous_drop_off: fields.get("continuous_drop_off")
                .filter(|s| !s.is_empty())
                .map(|s| s.parse::<routes::RouteContinuityPolicy>())
                .transpose()
                .map_err(StopTimeLoadError::ContinuousDropOffError)?,
            shape_dist_traveled: fields.get("shape_dist_traveled")
                .filter(|s| !s.is_empty())
                .map(|s| s.parse::<f64>())
                .transpose()
                .map_err(StopTimeLoadError::ShapeDistTraveledError)?,
            pickup_booking_rule_id: fields.get("pickup_booking_rule_id")
                .filter(|s| !s.is_empty())
                .cloned(),
            drop_off_booking_rule_id: fields.get("drop_off_booking_rule_id")
                .filter(|s| !s.is_empty())
                .cloned(),
            timepoint: fields.get("timepoint")
                .filter(|s| !s.is_empty())
                .map(|s| s.parse::<Timepoint>())
                .transpose()
                .map_err(StopTimeLoadError::TimepointError)?,
        })
    }
}

#[derive(Debug)]
pub enum ParseTimeError {
    ImproperNumberOfSegments,
    InvalidHourSegment(num::ParseIntError),
    InvalidMinuteSegment(num::ParseIntError),
    InvalidSecondSegment(num::ParseIntError),
    InvalidTime(u32, u32, u32),
}

impl fmt::Display for ParseTimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ImproperNumberOfSegments => write!(f, "Improper number of segments"),
            Self::InvalidHourSegment(e) => write!(f, "Invalid hour segment: {}", e),
            Self::InvalidMinuteSegment(e) => write!(f, "Invalid minute segment: {}", e),
            Self::InvalidSecondSegment(e) => write!(f, "Invalid second segment: {}", e),
            Self::InvalidTime(h, m, s) => write!(f, "Invalid time '{}:{}:{}'", h, m, s),
        }
    }
}

fn parse_time(s: &str) -> Result<chrono::NaiveTime, ParseTimeError> {
    let segments = s.split(':').collect::<Vec<&str>>();
    if segments.len() != 3 {
        return Err(ParseTimeError::ImproperNumberOfSegments);
    }
    let hours = segments[0].parse::<u32>().map_err(|e| ParseTimeError::InvalidHourSegment(e))? % 24;
    let minutes = segments[1].parse::<u32>().map_err(|e| ParseTimeError::InvalidMinuteSegment(e))?;
    let seconds = segments[2].parse::<u32>().map_err(|e| ParseTimeError::InvalidSecondSegment(e))?;
    chrono::NaiveTime::from_hms_opt(hours, minutes, seconds)
        .ok_or(ParseTimeError::InvalidTime(hours, minutes, seconds))
}