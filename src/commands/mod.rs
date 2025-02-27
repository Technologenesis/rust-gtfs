mod gtfs;

trait CommandInterpreter {
    type CommandResult;
    type CommandError: std::error::Error;

    fn interpret(&self, command: &str) -> Result<Self::CommandResult, Self::CommandError>;
    fn interpret_mut(&mut self, command: &str) -> Result<(), Self::CommandError>;
}