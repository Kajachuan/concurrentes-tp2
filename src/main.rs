use std::io;
use std::env;
use std::sync::{mpsc, Arc, Barrier};

mod coordinator;
mod table;
mod player;
mod outputer;

fn get_players_number() -> u32 {
    println!("Ingrese la cantidad de jugadores:");

    let mut players_number = String::new();
    io::stdin().read_line(&mut players_number).expect("Error al leer la entrada");

    let mut invalid_value = true;
    let mut current_value = players_number.trim().parse::<u32>();

    if current_value.is_ok() {
        let n = current_value.clone().unwrap();
        if n >= 4 && n % 2 == 0 {
            invalid_value = false;
        } else {
            invalid_value = true;
        }
    }

    while invalid_value {
        println!("El valor ingresado es inválido: debe ser un número par mayor o igual a 4.");
        println!("Ingrese la cantidad de jugadores:");

        players_number.clear();
        io::stdin().read_line(&mut players_number).expect("Error al leer la entrada");

        current_value = players_number.trim().parse::<u32>();
        if current_value.is_ok() {
            let n = current_value.clone().unwrap();
            if n >= 4 && n % 2 == 0 {
                invalid_value = false;
            } else {
                invalid_value = true;
            }
        }
    }

    return current_value.unwrap();
}

fn main() {
    let (tx_output, rx_output) = mpsc::channel::<String>();
    let outputer;
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && &args[1] == "debug" {
        outputer = outputer::init(true, rx_output);
    } else {
        outputer = outputer::init(false, rx_output);
    }

    // Cantidad de jugadores
    let players_number = get_players_number();

    // Barrier
    let barrier = Arc::new(Barrier::new(players_number as usize + 2));

    // Canal Coordinador -> Mesa
    let (tx_coord_table, rx_coord_table) = mpsc::channel::<String>();

    // Canal Mesa -> Coordinador
    let (tx_table_coord, rx_table_coord) = mpsc::channel::<String>();

    // Canal Jugadores -> Mesa
    let (tx_players_table, rx_players_table) = mpsc::channel::<String>();

    // Canales Mesa -> JugadorX
    let mut tx_table_player = Vec::new();
    let mut rx_table_player = Vec::new();
    for _ in 0..players_number {
        let (tx, rx) = mpsc::channel::<String>();
        tx_table_player.push(tx);
        rx_table_player.push(rx);
    }

    // JUGADORES
    let mut players = Vec::new();
    for player in 0..players_number {
        let rx_table_player = rx_table_player.remove(0);
        let tx_player_table = tx_players_table.clone();
        let tx_output_player = tx_output.clone();
        let player_barrier = barrier.clone();

        let new_player = player::init(player + 1, players_number, rx_table_player,
                                      tx_player_table, tx_output_player, player_barrier);
        players.push(new_player);
    }

    // MESA
    let table_barrier = barrier.clone();
    let table = table::init(players, rx_coord_table, tx_table_player,
                            rx_players_table, tx_table_coord, table_barrier);

    // COORDINADOR
    let tx_output_coordinator = tx_output.clone();
    let coordinator = coordinator::init(players_number, table, tx_coord_table,
                                        rx_table_coord, tx_output_coordinator, barrier);

    coordinator.join().unwrap();

    tx_output.send(String::from("END")).unwrap();
    outputer.join().unwrap();
}
