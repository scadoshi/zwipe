const SUCCESS_MESSAGES: [&str; 10] = [
    "bazinga! your data has been transmogrified into digital gold!",
    "eureka! the cosmic alignment of your request has been achieved!",
    "shazam! your magical incantation has summoned the perfect response!",
    "kapow! your data has been transformed with the power of a thousand suns!",
    "wham! your request has been processed with the force of a neutron star!",
    "boom! your data has been launched into the success stratosphere!",
    "ding! your submission has been accelerated to light speed!",
    "pow! your magical gathering has summoned the ultimate response!",
    "bam! your request has been processed with the precision of a quantum computer!",
    "zap! your submission has been electrified into digital perfection!",
];

use rand::Rng;

pub fn get_random_success_message() -> String {
    let mut rng = rand::rng();
    let index = rng.random_range(0..SUCCESS_MESSAGES.len());
    SUCCESS_MESSAGES[index].to_string()
}
