const SUCCESS_MESSAGES: [&str; 15] = [
    "bazinga!",
    "shazam!",
    "eureka!",
    "five booms!",
    "pow!",
    "excelsior!",
    "up up and away!",
    "beamed up!",
    "positively stupendous!",
    "success has been achieved!",
    "victory has been attained!",
    "dominance has been established!",
    "information transmission received!",
    "configuration transfer protocol sent!",
    "your request has ascended into the heavens!",
];

use rand::Rng;

pub fn random_success_message() -> String {
    let mut rng = rand::rng();
    let index = rng.random_range(0..SUCCESS_MESSAGES.len());
    SUCCESS_MESSAGES[index].to_string()
}
