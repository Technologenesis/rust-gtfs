use crate::commands;
use crate::gtfs::GtfsSchedule;
use crate::commands::stops;
use crate::commands::routes;
use crate::commands::trips;

#[derive(Debug, Clone)]
pub struct GtfsNode {
    pub gtfs: GtfsSchedule,
    pub parent: Option<Box<GtfsNode>>,
    pub node_id: String,
    pub node_name: Option<String>,

}

#[derive(Debug)]
pub enum GTFSCommandInterpreterError {
    InvalidCommand(String),
    StopsSubcommandRequired,
    StopsSubcommandError(Box<stops::StopsCommandError>),
    RoutesCommandError(routes::RoutesCommandError),
    TripsCommandError(trips::TripsCommandError),
}

impl std::fmt::Display for GTFSCommandInterpreterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GTFSCommandInterpreterError::InvalidCommand(command) => write!(f, "Invalid command: {}", command),
            GTFSCommandInterpreterError::StopsSubcommandError(e) => write!(f, "Error interpreting stops subcommand: {}", e),
            GTFSCommandInterpreterError::StopsSubcommandRequired => write!(f, "Stops subcommand required"),
            GTFSCommandInterpreterError::RoutesCommandError(e) => write!(f, "Error interpreting routes command: {}", e),
            GTFSCommandInterpreterError::TripsCommandError(e) => write!(f, "Error interpreting trips command: {}", e),
        }
    }
}

impl std::error::Error for GTFSCommandInterpreterError {}

impl commands::CommandInterpreter for GtfsNode {
    type CommandResult = ();
    type CommandError = GTFSCommandInterpreterError;

    fn interpret(&self, command: &str) -> Result<Self::CommandResult, Self::CommandError> {
        let (first, rest) = command.find(".").and_then(|i| command.split_at_checked(i)).unwrap_or((command, ""));
        match first {
            "info" => Ok(println!("{}", &self.gtfs)),
            "stops" => match try_tail(rest) {
                Some(tail) => stops::StopsCommandInterpreter(&self.gtfs)
                    .interpret(tail.as_str())
                    .map_err(|e| GTFSCommandInterpreterError::StopsSubcommandError(Box::new(e))),
                None => Err(GTFSCommandInterpreterError::StopsSubcommandRequired),
            },
            "routes" => routes::RoutesCommandInterpreter(&self)
                .interpret(String::from(&rest[1..]).as_str())
                .map_err(GTFSCommandInterpreterError::RoutesCommandError),
            "trips" => trips::TripsCommandInterpreter(&self.gtfs)
                .interpret(String::from(&rest[1..]).as_str())
                .map_err(GTFSCommandInterpreterError::TripsCommandError),
            _ => Err(GTFSCommandInterpreterError::InvalidCommand(command.to_string())),
        }
    }
}

fn try_tail(s: &str) -> Option<String> {
    let s = s.chars().skip(1).collect::<String>();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}
