use crate::commands;
use crate::gtfs;

#[derive(Debug)]
enum GTFSCommandInterpreterError {
    InvalidCommand(String),
    StopsCommandError(StopCommandError),
    RoutesCommandError(RouteCommandError),
    TripsCommandError(TripCommandError),
}

impl std::fmt::Display for GTFSCommandInterpreterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GTFSCommandInterpreterError::InvalidCommand(command) => write!(f, "Invalid command: {}", command),
            GTFSCommandInterpreterError::StopsCommandError(e) => write!(f, "{}", e),
            GTFSCommandInterpreterError::RoutesCommandError(e) => write!(f, "{}", e),
            GTFSCommandInterpreterError::TripsCommandError(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for GTFSCommandInterpreterError {}

impl commands::CommandInterpreter for gtfs::GtfsSchedule {
    type CommandResult = ();
    type CommandError = GTFSCommandInterpreterError;

    fn interpret(&self, command: &str) -> Result<Self::CommandResult, Self::CommandError> {
        let (first, rest) = command.find(".").and_then(|i| command.split_at_checked(i)).unwrap_or((command, ""));
        match first {
            "stops" => {
                let stop_id = rest.parse::<u32>().map_err(|_| GTFSCommandInterpreterError::InvalidCommand(command.to_string()))?;
                let stop = self.stops.get(&stop_id).ok_or(GTFSCommandInterpreterError::InvalidCommand(command.to_string()))?;
                Ok(())
            }
            "routes" => {
                let route_id = rest.parse::<u32>().map_err(|_| GTFSCommandInterpreterError::InvalidCommand(command.to_string()))?;
                let route = self.routes.get(&route_id).ok_or(GTFSCommandInterpreterError::InvalidCommand(command.to_string()))?;
                Ok(())
            }
            "trips" => {
                let trip_id = rest.parse::<u32>().map_err(|_| GTFSCommandInterpreterError::InvalidCommand(command.to_string()))?;
                let trip = self.trips.get(&trip_id).ok_or(GTFSCommandInterpreterError::InvalidCommand(command.to_string()))?;
                Ok(())
            }
            _ => Err(GTFSCommandInterpreterError::InvalidCommand(command.to_string())),
        }
    }

    fn interpret_mut(&mut self, command: &str) -> Result<(), Self::CommandError> {
        Ok(())
    }
}